use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use alloy_primitives::Address;
use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use x402_axum::X402Middleware;
use x402_chain_eip155::{KnownNetworkEip155, V1Eip155Exact};
use x402_types::networks::USDC;

use crate::api::{self, Market};
use crate::strategy;

const CACHE_TTL: Duration = Duration::from_secs(60);

struct AppState {
    cache: RwLock<Option<(Instant, Vec<Market>)>>,
}

async fn markets_cached(state: &AppState) -> Result<Vec<Market>> {
    if let Some((fetched, markets)) = state.cache.read().await.as_ref() {
        if fetched.elapsed() < CACHE_TTL {
            return Ok(markets.clone());
        }
    }
    let fresh = api::fetch_markets().await?;
    *state.cache.write().await = Some((Instant::now(), fresh.clone()));
    Ok(fresh)
}

struct Upstream(anyhow::Error);

impl IntoResponse for Upstream {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("upstream: {}", self.0) })),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for Upstream {
    fn from(e: anyhow::Error) -> Self {
        Upstream(e)
    }
}

#[derive(Serialize)]
struct RateEntry {
    coin: String,
    rate_hourly_pct: f64,
    apr_pct: f64,
    open_interest_usd: f64,
}

impl From<&Market> for RateEntry {
    fn from(m: &Market) -> Self {
        RateEntry {
            coin: m.coin.clone(),
            rate_hourly_pct: m.funding_rate * 100.0,
            apr_pct: m.funding_annual(),
            open_interest_usd: m.open_interest,
        }
    }
}

fn ranked(markets: &[Market], k: usize, min_oi: f64) -> (Vec<RateEntry>, Vec<RateEntry>) {
    let mut sorted: Vec<&Market> = markets.iter().filter(|m| m.open_interest >= min_oi).collect();
    sorted.sort_by(|a, b| a.funding_rate.partial_cmp(&b.funding_rate).unwrap());
    let longs = sorted.iter().take(k).map(|m| RateEntry::from(*m)).collect();
    let shorts = sorted.iter().rev().take(k).map(|m| RateEntry::from(*m)).collect();
    (longs, shorts)
}

#[derive(Deserialize)]
struct RatesQuery {
    #[serde(default = "default_k")]
    k: usize,
    #[serde(default)]
    min_oi: f64,
}

fn default_k() -> usize {
    5
}

async fn rates(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RatesQuery>,
) -> Result<Json<serde_json::Value>, Upstream> {
    let markets = markets_cached(&state).await?;
    let (longs, shorts) = ranked(&markets, q.k, q.min_oi);
    Ok(Json(serde_json::json!({
        "markets_total": markets.len(),
        "note": "negative funding: longs collect hourly; positive: shorts collect",
        "long_candidates": longs,
        "short_candidates": shorts,
    })))
}

#[derive(Deserialize)]
struct BasketQuery {
    #[serde(default = "default_capital")]
    capital: f64,
    #[serde(default = "default_k")]
    k: usize,
    #[serde(default = "default_min_oi")]
    min_oi: f64,
}

fn default_capital() -> f64 {
    10_000.0
}

fn default_min_oi() -> f64 {
    5_000_000.0
}

#[derive(Serialize)]
struct Leg {
    #[serde(flatten)]
    entry: RateEntry,
    side: &'static str,
    notional_usd: f64,
}

async fn basket(
    State(state): State<Arc<AppState>>,
    Query(q): Query<BasketQuery>,
) -> Result<Json<serde_json::Value>, Upstream> {
    let markets = markets_cached(&state).await?;
    let filtered: Vec<Market> = markets
        .into_iter()
        .filter(|m| m.open_interest >= q.min_oi)
        .collect();
    let snap = strategy::scan(&filtered, q.k, q.capital);
    let per_side = q.capital / q.k as f64;
    let leg = |m: &Market, side: &'static str| Leg {
        entry: RateEntry::from(m),
        side,
        notional_usd: per_side,
    };
    Ok(Json(serde_json::json!({
        "capital_usd": snap.capital,
        "k": q.k,
        "min_oi_usd": q.min_oi,
        "net_funding_daily_usd": snap.net_funding_daily,
        "net_funding_annual_pct": snap.net_funding_annual_pct,
        "legs": snap.longs.iter().map(|m| leg(m, "long"))
            .chain(snap.shorts.iter().map(|m| leg(m, "short")))
            .collect::<Vec<_>>(),
        "disclaimer": "raw funding edge; momentum/rebalance costs not modeled — backtest before trading",
    })))
}

async fn preview(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, Upstream> {
    let markets = markets_cached(&state).await?;
    let (longs, shorts) = ranked(&markets, 1, 0.0);
    Ok(Json(serde_json::json!({
        "top_long": longs.first(),
        "top_short": shorts.first(),
        "full_ranking": "GET /rates — x402 payment required",
    })))
}

async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "hyperfund — Hyperliquid funding-rate signals, payable by AI agents (x402)",
        "source": "https://github.com/bigrender/hyperfund",
        "endpoints": {
            "GET /preview": { "price": "free", "desc": "top-1 long/short funding edge" },
            "GET /rates?k=&min_oi=": { "price_usdc": price_env("X402_PRICE_RATES", "0.005"), "desc": "full funding ranking" },
            "GET /basket?capital=&k=&min_oi=": { "price_usdc": price_env("X402_PRICE_BASKET", "0.02"), "desc": "delta-neutral basket plan" },
        },
        "payment": "x402 exact scheme, USDC. Unpaid requests receive 402 with requirements.",
    }))
}

fn price_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// "0.005" (USDC decimal) → 5000 (atomic, 6 decimals)
fn usdc_atomic(price: &str) -> Result<u64> {
    let v: f64 = price.parse().with_context(|| format!("invalid USDC price: {price}"))?;
    anyhow::ensure!(v > 0.0 && v < 1_000_000.0, "price out of range: {price}");
    Ok((v * 1_000_000.0).round() as u64)
}

pub async fn serve(port: u16, pay_to: &str) -> Result<()> {
    let pay_to = Address::from_str(pay_to).context("invalid --pay-to address (expect 0x…40 hex)")?;
    let facilitator =
        std::env::var("X402_FACILITATOR").unwrap_or_else(|_| "https://facilitator.x402.rs".into());
    let network = std::env::var("X402_NETWORK").unwrap_or_else(|_| "base".into());
    let usdc = match network.as_str() {
        "base-sepolia" => USDC::base_sepolia(),
        _ => USDC::base(),
    };
    let price_rates = price_env("X402_PRICE_RATES", "0.005");
    let price_basket = price_env("X402_PRICE_BASKET", "0.02");

    // 402 응답의 resource URI는 클라이언트가 보는 공개 URL이어야 한다 (배포 시 필수)
    let mut x402 = X402Middleware::new(&facilitator);
    if let Ok(base) = std::env::var("X402_BASE_URL") {
        x402 = x402.with_base_url(base.parse().context("invalid X402_BASE_URL")?);
    }
    let rates_amount = usdc.amount(usdc_atomic(&price_rates)?);
    let basket_amount = usdc.amount(usdc_atomic(&price_basket)?);

    let state = Arc::new(AppState { cache: RwLock::new(None) });

    let app = Router::new()
        .route("/", get(root))
        .route("/preview", get(preview))
        .route(
            "/rates",
            get(rates).layer(x402.with_price_tag(V1Eip155Exact::price_tag(pay_to, rates_amount))),
        )
        .route(
            "/basket",
            get(basket).layer(x402.with_price_tag(V1Eip155Exact::price_tag(pay_to, basket_amount))),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;

    println!(
        "hyperfund serve :{port} · network {network} · pay-to {pay_to} · /rates ${price_rates} /basket ${price_basket}"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(coin: &str, rate: f64, oi: f64) -> Market {
        Market { coin: coin.into(), funding_rate: rate, open_interest: oi }
    }

    #[test]
    fn usdc_atomic_converts_and_rejects_junk() {
        assert_eq!(usdc_atomic("0.005").unwrap(), 5_000);
        assert_eq!(usdc_atomic("0.02").unwrap(), 20_000);
        assert_eq!(usdc_atomic("1").unwrap(), 1_000_000);
        assert!(usdc_atomic("0").is_err());
        assert!(usdc_atomic("-1").is_err());
        assert!(usdc_atomic("free").is_err());
    }

    #[test]
    fn ranked_puts_most_negative_first_and_filters_thin_markets() {
        let markets = vec![
            mk("THIN", -0.9, 1_000.0),
            mk("NEG", -0.01, 20_000_000.0),
            mk("FLAT", 0.0, 20_000_000.0),
            mk("POS", 0.02, 20_000_000.0),
        ];
        let (longs, shorts) = ranked(&markets, 2, 10_000_000.0);
        assert_eq!(longs[0].coin, "NEG", "long candidate = most negative funding");
        assert_eq!(shorts[0].coin, "POS", "short candidate = most positive funding");
        assert!(!longs.iter().any(|e| e.coin == "THIN"), "min_oi must filter thin markets");
    }

    #[test]
    fn apr_matches_hourly_rate() {
        let e = RateEntry::from(&mk("X", 0.0001, 1.0));
        assert!((e.rate_hourly_pct - 0.01).abs() < 1e-9);
        assert!((e.apr_pct - 0.0001 * 24.0 * 365.0 * 100.0).abs() < 1e-9);
    }
}

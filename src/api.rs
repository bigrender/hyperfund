use serde::Serialize;
use anyhow::{Context, Result};

const HL_API: &str = "https://api.hyperliquid.xyz/info";

#[derive(Debug, Clone)]
pub struct Market {
    pub coin: String,
    pub funding_rate: f64,   // per hour, raw
    pub open_interest: f64,  // USD
}

impl Market {
    pub fn funding_annual(&self) -> f64 {
        self.funding_rate * 24.0 * 365.0 * 100.0 // %
    }
}

#[derive(Serialize)]
struct InfoRequest {
    #[serde(rename = "type")]
    req_type: &'static str,
}

pub async fn fetch_markets() -> Result<Vec<Market>> {
    let client = reqwest::Client::new();
    let resp = client
        .post(HL_API)
        .json(&InfoRequest { req_type: "metaAndAssetCtxs" })
        .send()
        .await
        .context("HL API 연결 실패")?;

    let raw: serde_json::Value = resp.json().await.context("응답 파싱 실패")?;

    let universe = raw[0]["universe"]
        .as_array()
        .context("universe 없음")?;
    let ctxs = raw[1]
        .as_array()
        .context("assetCtxs 없음")?;

    let mut markets = Vec::new();
    for (u, c) in universe.iter().zip(ctxs.iter()) {
        let coin = u["name"].as_str().unwrap_or("?").to_string();
        let funding_rate: f64 = c["funding"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let mark_px: f64 = c["markPx"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
        let oi_coins: f64 = c["openInterest"].as_str().unwrap_or("0").parse().unwrap_or(0.0);

        if mark_px <= 0.0 {
            continue;
        }

        markets.push(Market {
            coin,
            funding_rate,
            open_interest: oi_coins * mark_px,
        });
    }

    Ok(markets)
}

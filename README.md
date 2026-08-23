# hyperfund

**Hyperliquid has 230+ perp markets. Some of them are paying you 20%+ APR just to hold a position right now. This tool shows you which ones.**

One binary. No API key. Live funding data every run.

📊 **[Live funding snapshot →](./SNAPSHOT.md)** — auto-refreshed every 6 hours.

Example run (`rates --min-oi 10000000`, real snapshot — funding moves every hour):

```
  HYPERFUND  ·  Hyperliquid Funding Rates
  ─────────────────────────────────────────────────────────────

  LONG CANDIDATES  (negative funding → you collect)
  COIN            RATE/HR          APR       OI (USD)
  ─────────────────────────────────────────────────────────────
  TRUMP          -0.0226%      -197.6%         $11.6M
  TON            -0.0049%       -43.3%         $38.7M
  TRX            -0.0026%       -22.5%         $18.8M

  SHORT CANDIDATES  (positive funding → you collect)
  COIN            RATE/HR          APR       OI (USD)
  ─────────────────────────────────────────────────────────────
  XMR            +0.0089%       +78.2%         $42.0M
  LIT            +0.0013%       +11.0%         $60.8M
  MON            +0.0013%       +11.0%         $26.4M
```

## Why

Hyperliquid pays funding every hour. Long positions in negative-funding markets receive the rate; short positions in positive-funding markets receive the rate. A dollar-neutral basket capturing both sides earns the spread with no directional exposure — in theory.

In practice, the basket is not perfectly market-neutral: shorting high-funding (crowded long) coins and longing low-funding (crowded short) coins introduces adverse momentum exposure. This tool surfaces the raw funding edge. A full backtest is needed before trading.

A companion Python backtesting engine quantifies the funding vs. price residual decomposition. The Rust binary here is the live scanning companion.

## Install

```bash
git clone <this repo>
cd hyperfund
cargo build --release
./target/release/hyperfund --help
```

Requires Rust 1.75+. No API key needed — read-only public endpoints only.

## Usage

### Show funding rates
```bash
# Top 5 long/short candidates across all markets
hyperfund rates

# Filter to liquid markets only (≥$10M OI)
hyperfund rates --min-oi 10000000

# Watch mode: refresh every 60 seconds
hyperfund rates --watch 60

# Show top 10 per side
hyperfund rates --k 10
```

### Delta-neutral scan
```bash
# Estimate funding yield for a $10,000 dollar-neutral basket (K=5)
hyperfund scan --capital 10000

# With liquidity filter (recommended for real trading)
hyperfund scan --capital 10000 --k 5 --min-oi 10000000
```

Output includes estimated `$/day` and `% APR` from funding alone, with an explicit warning that price residual is excluded. Use the backtest to model the full P&L.

## Agent API (x402)

**Your AI agent can pay half a cent for a funding edge. No API key, no signup, no subscription.**

The same engine also runs as an HTTP service where AI agents settle in USDC per request over [x402](https://docs.cdp.coinbase.com/x402/welcome) — the HTTP 402 payment protocol. CoinGecko sells prices this way; hyperfund sells edges.

```bash
hyperfund serve --port 8080 --pay-to 0xYourBaseAddress
```

| Endpoint | Price | Returns |
|---|---|---|
| `GET /` | free | service description + live prices |
| `GET /preview` | free | top-1 long/short funding edge |
| `GET /rates?k=&min_oi=` | $0.005 USDC | full funding ranking |
| `GET /basket?capital=&k=&min_oi=` | $0.02 USDC | delta-neutral basket plan with per-leg sizing |

An unpaid request to a priced route returns `402` with the payment requirements; any x402 client library handles the retry automatically.

```bash
# free — no wallet needed
curl https://your-host/preview

# paid — 402 first, then the agent's x402 client attaches the USDC authorization
curl -i https://your-host/rates?k=8&min_oi=10000000
```

Settlement is USDC on Base via the `exact` scheme. Configure with `X402_PRICE_RATES`, `X402_PRICE_BASKET`, `X402_NETWORK` (`base` | `base-sepolia`), `X402_FACILITATOR`, and `X402_BASE_URL` (required in production — it sets the public resource URI in the 402 response).

## Architecture

```
src/
├── main.rs      — CLI entry (clap), command dispatch
├── api.rs       — Hyperliquid Info API (metaAndAssetCtxs, no SDK dependency)
├── strategy.rs  — Dollar-neutral basket construction, funding yield estimate
├── server.rs    — x402-paid HTTP API (axum), 60s market cache
└── display.rs   — Colored terminal output
```

Single binary, ~3.7MB release build. No runtime dependencies. Pure async Rust (tokio + reqwest).

## Roadmap

- [ ] WebSocket real-time streaming (vs. poll)
- [ ] Trade execution with builder code (revenue-share on every fill)
- [ ] Beta-neutralized basket (hedge BTC beta to isolate funding alpha)
- [ ] Equity curve visualization

## License

MIT

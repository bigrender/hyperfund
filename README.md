# hyperfund

**Hyperliquid has 230+ perp markets. Some of them are paying you 20%+ APR just to hold a position right now. This tool shows you which ones.**

One binary. No API key. Live funding data every run.

```
  HYPERFUND  ·  Hyperliquid Funding Rates
  12:42:53 UTC
  ─────────────────────────────────────────────────────────────

  LONG CANDIDATES  (negative funding → you collect)
  COIN            RATE/HR          APR       OI (USD)
  ─────────────────────────────────────────────────────────────
  TON            -0.0019%       -16.3%         $34.2M
  BTC            -0.0007%        -6.6%          $1.2B
  XRP            -0.0004%        -3.8%         $180.1M

  SHORT CANDIDATES  (positive funding → you collect)
  COIN            RATE/HR          APR       OI (USD)
  ─────────────────────────────────────────────────────────────
  ZRO            +0.0022%       +19.4%         $34.7M
  XMR            +0.0013%       +11.0%         $12.3M
```

## Why

Hyperliquid pays funding every hour. Long positions in negative-funding markets receive the rate; short positions in positive-funding markets receive the rate. A dollar-neutral basket capturing both sides earns the spread with no directional exposure — in theory.

In practice, the basket is not perfectly market-neutral: shorting high-funding (crowded long) coins and longing low-funding (crowded short) coins introduces adverse momentum exposure. This tool surfaces the raw funding edge. A full backtest is needed before trading.

The [`backtest/`](../backtest/) directory contains the Python backtesting engine that quantifies the funding vs. price residual decomposition. The Rust binary here is the live scanning companion.

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

## Architecture

```
src/
├── main.rs      — CLI entry (clap), command dispatch
├── api.rs       — Hyperliquid Info API (metaAndAssetCtxs, no SDK dependency)
├── strategy.rs  — Dollar-neutral basket construction, funding yield estimate
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

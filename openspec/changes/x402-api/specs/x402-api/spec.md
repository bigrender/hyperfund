# x402-api Delta

## ADDED Requirements

### Requirement: Paid rates endpoint

The system SHALL serve `GET /rates` behind x402 payment middleware, priced at 0.005 USDC per call on Base, returning the funding-rate ranking as JSON (coin, hourly rate, APR, open interest, long/short side) reusing the existing engine in `api.rs`/`strategy.rs`.

#### Scenario: Unpaid request receives 402

- WHEN a client calls `GET /rates` without an `X-PAYMENT` header
- THEN the server responds `402 Payment Required` with x402 payment requirements (scheme `exact`, USDC asset on Base, amount 5000 atomic units, payTo set to the operator wallet)

#### Scenario: Paid request returns data

- WHEN a client retries `GET /rates` with a valid `X-PAYMENT` authorization verified by the facilitator
- THEN the server responds `200` with the funding ranking JSON and settles the payment via the facilitator

### Requirement: Paid basket endpoint

The system SHALL serve `GET /basket?capital=<usd>&k=<n>&min_oi=<usd>` behind the same x402 middleware, priced at 0.02 USDC per call, returning the delta-neutral basket plan JSON produced by `strategy::scan`.

#### Scenario: Basket plan for given capital

- WHEN a paying client calls `GET /basket?capital=10000&k=5`
- THEN the response contains K long and K short legs with per-leg notional sizing summing to the requested capital

### Requirement: Free discovery surface

The system SHALL serve `GET /` (service description, endpoint list, prices) and `GET /preview` (top-1 funding edge only) without payment, so agents and humans can evaluate before paying.

#### Scenario: Preview is free but limited

- WHEN any client calls `GET /preview` without payment
- THEN the server responds `200` with exactly one long candidate and one short candidate and a pointer to the paid `/rates` endpoint

### Requirement: Serve subcommand

The system SHALL expose the server via a `hyperfund serve --port <p> --pay-to <address>` subcommand; existing CLI subcommands (`rates`, `scan`, watch mode) SHALL remain unchanged.

#### Scenario: CLI unaffected

- WHEN a user runs `hyperfund rates`
- THEN behavior and output are identical to the pre-change binary

# Design: x402-api

## Context

hyperfund는 319줄짜리 Rust CLI (api.rs: Hyperliquid fetch → strategy.rs: 랭킹/바스켓 → display.rs: 터미널 출력). 서버는 display만 JSON 직렬화로 바꾸면 되는 구조. 2026-08-22 검증된 외부 사실은 proposal.md 참조.

## Decisions

### 스택: axum + x402-axum

- `x402-axum` v2.0.2 (github.com/x402-rs/x402-rs): tower layer로 라우트에 가격을 붙이는 방식. 이미 tokio 사용 중이라 추가 의존성은 axum 계열뿐.
- 대안 검토: 수동 402 구현 — 미들웨어가 검증/정산/재시도 처리를 다 해주므로 기각.

### Facilitator: CDP (Coinbase)

- 검증 무료, 월 1,000건 무료 정산 후 $0.001/건. CDP API 키 필요 (기존 Coinbase 계정으로 발급).
- 대안: x402.org 커뮤니티 facilitator — Bazaar 등록이 CDP 쪽과 묶여 있어 CDP 우선. 전환 비용은 env var 하나.

### 가격: /rates $0.005, /basket $0.02

- 근거: CoinGecko 원시 데이터 $0.01의 절반(우린 무명) / 가공 시그널은 kronossignals $0.02와 동급.
- 402 응답이 가격의 단일 소스이므로 env var로 조정 가능하게 (`X402_PRICE_RATES`, `X402_PRICE_BASKET`). 하드코딩 금지 — 시장 테스트에서 가격 실험이 예정되어 있음.

### 상태 없음, DB 없음

- 결제 기록은 온체인 + facilitator가 보관. 서버는 무상태 → Fly.io 1대면 충분. 캐싱은 Hyperliquid API 응답 60초 in-memory 1개면 끝 (이미 watch 모드에 유사 패턴 존재).

## Risks

- 시장이 아직 작음 (proposal의 ~$325/월 상한 참조) → 시장 테스트 기준 미달 시 중단 조건을 proposal에 명시함
- GENIUS Act 시행규칙(2027-01 발효)에서 API 판매자의 지위는 현재 불명확 — 데이터 판매(비수탁, 비중개)라 리스크 최저 계층이지만 모니터링 필요
- Hyperliquid API 레이트리밋: 60초 캐시로 상쇄, 유료 호출량이 캐시 TTL을 넘게 성장하면 그때 대응 (좋은 문제)

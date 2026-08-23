# Change: x402-api — 에이전트가 USDC로 결제하는 Hyperliquid 시그널 API

## Why

hyperfund는 이미 Hyperliquid 펀딩 엣지를 계산하는 엔진(api.rs + strategy.rs)을 갖고 있지만 수익 모델이 없다. 2026-08 검증 결과(1차 소스):

- x402 Bazaar 디스커버리 API에 등록된 상위 100개 서비스 전원이 최근 30일 실사용 트래픽 보유 (2026-08-22 직접 쿼리)
- CoinGecko가 $0.01 USDC/콜 pay-per-use를 이미 운영 — per-query 과금 선례 확립
- 트레이딩 시그널 카테고리는 사실상 공백: 유사 서비스는 kronossignals(청산맵, $0.02/콜) 1개뿐
- Rust 생태계 준비 완료: x402-axum v2.0.2 미들웨어(다운로드 3.7만), CDP facilitator 검증 무료 + 월 1,000건 무료 정산 후 건당 $0.001
- 단, 현재 시장 규모는 작음: Bazaar 최다 호출 서비스도 월 매출 ~$325 수준. 이 변경은 큰 수익이 아니라 "성장 초기 시장 선점 + repo 어텐션" 베팅이다.

## What Changes

- `hyperfund serve` 서브커맨드 추가: 기존 엔진을 axum HTTP 서버로 노출
- 유료 엔드포인트 2개 (x402-axum 미들웨어, USDC on Base, `exact` 스킴):
  - `GET /rates` — 펀딩 레이트 랭킹 JSON ($0.005/콜)
  - `GET /basket` — 델타뉴트럴 바스켓 계획 JSON ($0.02/콜)
- 무료 엔드포인트: `GET /` (서비스 설명 + 가격), `GET /preview` (상위 1개만, 마케팅용 미끼)
- x402 Bazaar 디스커버리 등록으로 에이전트 검색 노출
- CLI 기존 동작(`rates`/`scan`/`watch`)은 변경 없음

## MDD

훅 (X/Farcaster/GitHub용):
- "Your AI agent just paid half a cent for a Hyperliquid funding edge. No API key, no signup, no subscription."
- "The first trading-signal API on x402 Bazaar with a delta-neutral basket endpoint. CoinGecko sells prices — we sell edges."
- "230+ perp markets, some paying 20%+ APR. Now queryable by any agent for $0.005 in USDC."
- "One Rust binary is now both a CLI and a paid API. Same engine, two products."

분배 가설: x402 Bazaar 디스커버리(에이전트가 자동 발견) + Farcaster/CT 런칭 포스트 + 기존 hyperfund repo(README에 API 섹션 추가)가 랜딩 역할. 공유 동기: "에이전트가 스스로 돈 내고 시그널 사는" 데모는 그 자체로 리트윗 소재.

시장 테스트 (이게 나와야 더 만든다): 배포 후 2주 내 유료 호출 100건 AND 유니크 페이어 5+ (Bazaar quality 지표 / payTo 주소 온체인 USDC 수신으로 측정). 미달 시 가격·상품 조정 또는 중단 — 기능 추가 금지.

## Impact

- Affected specs: x402-api (신규)
- Affected code: src/main.rs (serve 서브커맨드), src/server.rs (신규), Cargo.toml (axum, x402-axum 추가)
- 신규 운영 요소: CDP API 키(facilitator), 수신용 Base 지갑 주소, 상시 호스팅 1대 (Fly.io/Railway)
- CLI 사용자 영향 없음

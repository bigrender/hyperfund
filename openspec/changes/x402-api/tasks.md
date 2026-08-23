# Tasks: x402-api

## 1. 서버 코어

- [x] 1.1 Cargo.toml에 axum + x402-axum 추가, `hyperfund serve --port --pay-to` 서브커맨드 스캐폴드
- [x] 1.2 src/server.rs: `GET /` (설명+가격), `GET /preview` (top-1 long/short) — 무료 라우트
- [x] 1.3 `GET /rates` JSON 직렬화 (strategy 재사용), x402 미들웨어 $0.005 적용
- [x] 1.4 `GET /basket?capital&k&min_oi` JSON, x402 미들웨어 $0.02 적용
- [x] 1.5 Hyperliquid 응답 60초 in-memory 캐시
- [x] 1.6 셀프체크: 로컬 402 플로우 검증 완료 (무료 200 / 유료 402 + 정확한 결제요구). 실결제 200은 지갑 필요 → 2.1 이후

## 2. 운영 준비

- [ ] 2.1 수신용 Base 지갑 주소 생성(운영자 확인 필요), CDP API 키 발급
- [ ] 2.2 Fly.io 배포 + `hyperfund serve` 상시 실행, /preview 응답으로 헬스체크
- [ ] 2.3 x402 Bazaar 디스커버리 등록, 목록 노출 확인 (discovery API 재쿼리)

## 3. 런칭 (MDD)

- [ ] 3.1 README에 "Agent API" 섹션 추가 (훅 + curl 예시 + 가격표)
- [ ] 3.2 SNAPSHOT.md 하단에 API 배너 한 줄 (6시간마다 갱신되는 무료 광고판)
- [ ] 3.3 Farcaster/X 런칭 포스트: 에이전트가 USDC 내고 시그널 사는 30초 데모
- [ ] 3.4 시장 테스트 계측: payTo 주소 USDC 수신 + Bazaar quality 지표 확인 스크립트, 2주 후 판정 (100콜 AND 5 페이어)

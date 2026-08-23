#!/usr/bin/env bash
# hyperfund 시장 테스트 판정: x402 수신 지갑의 USDC 입금을 세어 기준 충족 여부를 출력한다.
# 기준 (openspec/changes/x402-api/proposal.md): 배포 2주 내 유료 100콜 AND 유니크 페이어 5+
set -euo pipefail

PAY_TO="${1:-}"
DAYS="${2:-14}"

if [[ -z "$PAY_TO" ]]; then
  cat <<'EOF'
usage: market-test.sh <payTo-address> [days=14]

  x402 수신 지갑에 들어온 Base USDC 입금을 조회해 시장 테스트 기준을 판정한다.
  기준: 유료 콜 100건 AND 유니크 페이어 5명 (배포 후 2주)

  예: ./scripts/market-test.sh 0xYourBaseAddress 14
  RPC 변경: BASE_RPC=https://... ./scripts/market-test.sh 0x...
EOF
  exit 1
fi

RPC="${BASE_RPC:-https://mainnet.base.org}"
USDC="0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"
BLOCKS_PER_DAY=43200          # Base 2초 블록
CHUNK="${CHUNK:-9000}"        # 공개 RPC의 eth_getLogs 범위 제한 회피

export PAY_TO DAYS RPC USDC BLOCKS_PER_DAY CHUNK
python3 <<'PY'
import json, os, subprocess, sys

rpc, usdc, pay_to = os.environ["RPC"], os.environ["USDC"], os.environ["PAY_TO"].lower()
days, per_day, chunk = int(os.environ["DAYS"]), int(os.environ["BLOCKS_PER_DAY"]), int(os.environ["CHUNK"])
TRANSFER = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"

# curl로 호출한다: python의 SSL 신뢰 저장소가 macOS에서 비어 있는 경우가 흔하다
def call(method, params):
    payload = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
    out = subprocess.run(
        ["curl", "-s", "-m", "30", "-H", "content-type: application/json", "-d", payload, rpc],
        capture_output=True, text=True, check=True).stdout
    body = json.loads(out)
    if "error" in body:
        raise RuntimeError(body["error"])
    return body["result"]

head = int(call("eth_blockNumber", []), 16)
start = max(0, head - days * per_day)
topic_to = "0x" + pay_to[2:].rjust(64, "0")

payers, transfers = {}, 0
lo = start
while lo <= head:
    hi = min(lo + chunk, head)
    try:
        logs = call("eth_getLogs", [{"fromBlock": hex(lo), "toBlock": hex(hi),
                                     "address": usdc, "topics": [TRANSFER, None, topic_to]}])
    except Exception as e:
        print(f"  RPC 오류 (블록 {lo}-{hi}): {e}", file=sys.stderr)
        lo = hi + 1
        continue
    for log in logs:
        sender = "0x" + log["topics"][1][-40:]
        payers[sender] = payers.get(sender, 0) + 1
        transfers += 1
    lo = hi + 1

CALL_TARGET, PAYER_TARGET = 100, 5
calls_ok, payers_ok = transfers >= CALL_TARGET, len(payers) >= PAYER_TARGET

print(f"수신 지갑 : {pay_to}")
print(f"기간      : 최근 {days}일 (블록 {start}–{head})")
print(f"유료 콜   : {transfers} / {CALL_TARGET}  {'✅' if calls_ok else '❌'}")
print(f"유니크    : {len(payers)} / {PAYER_TARGET}  {'✅' if payers_ok else '❌'}")
if payers:
    top = sorted(payers.items(), key=lambda kv: -kv[1])[:5]
    print("상위 페이어: " + ", ".join(f"{a[:10]}…({n})" for a, n in top))
print()
if calls_ok and payers_ok:
    print("판정: PASS — 계속 만든다 (다음: 가격 실험 또는 엔드포인트 추가)")
    sys.exit(0)
print("판정: FAIL — 기능 추가 금지. 가격·훅·분배를 바꾸거나 중단한다.")
sys.exit(2)
PY

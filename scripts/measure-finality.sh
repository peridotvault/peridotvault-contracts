#!/usr/bin/env bash
# ANT-487 finality-lag measurement (testnet/localnet RPCs ONLY — never mainnet).
# Usage: RPC_URL=<https-rpc> ./scripts/measure-finality.sh [samples] [eip155|solana]
# Prints "<chain> latest=<n> finalized=<m> lag=<k>" lines for the doc table.
set -euo pipefail
RPC_URL="${RPC_URL:?set RPC_URL to an http(s) testnet/localnet RPC endpoint}"
MODE="${2:-eip155}"
N="${1:-5}"
case "$RPC_URL" in
  *mainnet*|*eth-mainnet*|*api.mainnet*) echo "REFUSED: mainnet-looking RPC" >&2; exit 1;;
esac
for i in $(seq 1 "$N"); do
  if [ "$MODE" = "solana" ]; then
    LATEST=$(curl -s -X POST "$RPC_URL" -H 'content-type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[{"commitment":"processed"}]}' | grep -o '"result":[0-9]*' | grep -o '[0-9]*')
    FINAL=$(curl -s -X POST "$RPC_URL" -H 'content-type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"getSlot","params":[{"commitment":"finalized"}]}' | grep -o '"result":[0-9]*' | grep -o '[0-9]*')
    echo "solana latest=$LATEST finalized=$FINAL lag_slots=$((LATEST - FINAL))"
  else
    LATEST=$(curl -s -X POST "$RPC_URL" -H 'content-type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"eth_getBlockByNumber","params":["latest",false]}' | grep -o '"number":"0x[0-9a-f]*"' | head -1)
    FINAL=$(curl -s -X POST "$RPC_URL" -H 'content-type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"eth_getBlockByNumber","params":["finalized",false]}' | grep -o '"number":"0x[0-9a-f]*"' | head -1)
    echo "eip155 latest=$LATEST finalized=$FINAL"
  fi
  sleep 2
done

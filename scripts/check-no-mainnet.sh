#!/usr/bin/env bash
# ANT-490 AC-FIX-1 CI gate: fixtures must carry zero mainnet identifiers and
# zero secret material. Fails closed (non-zero exit) on any hit.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAIL=0

hit() { echo "MAINNET-GATE FAIL: $1"; FAIL=1; }

# 1. No mainnet chain IDs in fixtures (exact eip155:1, not eip155:13xxx).
if grep -rnE '"eip155:1([^0-9]|$)' "$ROOT/fixtures" ; then hit "eip155 mainnet id in fixtures"; fi
# 2. No Solana mainnet genesis hash anywhere in fixtures.
if grep -rn '5eykt4UsFv8P8NJdTREpYbfmoaDMG5Uf2n4MWY5auP' "$ROOT/fixtures"; then hit "solana mainnet genesis in fixtures"; fi
# 3. No mainnet word in a live-address context inside fixtures.
if grep -rni 'mainnet' "$ROOT/fixtures" | grep -vi 'zero mainnet\|no mainnet\|mainnet genesis\|mainnet id' ; then hit "suspicious mainnet mention in fixtures"; fi
# 4. No secret-looking material in fixtures (mnemonics/private keys/RPC URLs).
if grep -rniE 'mnemonic|private[_-]?key.{0,8}[0-9a-f]{32,}|BEGIN .*PRIVATE KEY|https?://[^ ]*(infura|alchemy|quicknode)' "$ROOT/fixtures"; then hit "secret material in fixtures"; fi
# 5. Testnet fixture placeholders must stay labeled fixtureOnly.
if ! grep -q '"fixtureOnly": true' "$ROOT/fixtures/registry.testnets.json"; then hit "registry.testnets.json lost fixtureOnly flag"; fi

if [ "$FAIL" -eq 0 ]; then echo "MAINNET-GATE OK: fixtures carry zero mainnet IDs and zero secrets"; fi
exit "$FAIL"

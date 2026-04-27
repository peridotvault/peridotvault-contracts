#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT"

# Program keypairs under target/deploy are intentionally preserved. They define
# the checked-in program IDs and should not be regenerated accidentally.
rm -rf \
  programs/pgl1/target \
  programs/registry/target \
  programs/game-store/target \
  target/idl \
  target/client \
  target/profile \
  target/.quasar-last-size

find target/deploy -maxdepth 1 -type f -name '*.so' -delete 2>/dev/null || true
find target/deploy -maxdepth 1 -type f -name 'game_store-*' -delete 2>/dev/null || true

echo "cleaned generated Quasar targets; preserved target/deploy/*-keypair.json"

#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROGRAMS=(
  "programs/pgl1"
  "programs/registry"
  "programs/game-store"
)

cd "$ROOT"

rm -f "$ROOT/target/deploy/game_store.so" "$ROOT/target/deploy/game_store-keypair.json"

for program in "${PROGRAMS[@]}"; do
  echo "==> quasar build $program"
  (cd "$ROOT/$program" && quasar build "$@")
done

echo "==> generate consolidated IDL and clients"
rm -rf "$ROOT/target/idl" "$ROOT/target/client"
for program in "${PROGRAMS[@]}"; do
  quasar idl "$ROOT/$program"
done

echo "==> clean program-local generated targets"
for program in "${PROGRAMS[@]}"; do
  rm -rf "$ROOT/$program/target"
done

echo "==> done"

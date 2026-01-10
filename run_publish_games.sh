#!/usr/bin/env bash
set -euo pipefail

echo "[publish_5_games] Publishing 5 games via Factory on localhost..."
npx hardhat run scripts/publish-5-games.ts --network localhost
echo "[publish_5_games] Done."

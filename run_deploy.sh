npx hardhat clean

set -euo pipefail

echo "[run_test] Compile..."
npx hardhat compile

echo "[run_test] Deploy local system (PGC1 impl + registry + factory)..."
npx hardhat run scripts/deploy-local-system.ts --network localhost

echo "[run_test] Done. Check deployments/localhost.json"

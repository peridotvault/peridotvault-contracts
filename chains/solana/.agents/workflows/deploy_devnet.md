---
description: How to deploy the PGC1 ecosystem to Solana Devnet
---

# PGC1 Devnet Deployment Workflow

Follow these steps to deploy and initialize the Registry, PGC1, and Game Store programs on Solana Devnet.

### 1. Environment Setup
Switch your Solana CLI to Devnet:
```bash
solana config set --url https://api.devnet.solana.com
```

### 2. Fund Deployment Wallet
Ensure your wallet has at least 5-10 SOL (deployment and account rent):
// turbo
```bash
# Repeat if balance is insufficient
solana airdrop 2
```

### 3. Update Configuration
In `Anchor.toml`, ensure the cluster is set to devnet:
```toml
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

### 4. Build & Deploy
Sync program IDs and deploy all programs:
// turbo
```bash
anchor build
anchor deploy
```

### 5. Initialize Registry
Call the initialize instruction on the Registry program:
```bash
# This sets the platform authority
anchor run initialize-registry
```

### 6. Initialize Game Store
Call the initialize instruction on the Game Store program:
```bash
# This sets the treasury and platform fee (e.g., 500 bps = 5%)
anchor run initialize-store
```

### 7. Verify with Console
Test the deployment using the unified create-game flow:
```bash
node app/console.cjs
```

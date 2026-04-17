---
description: How to deploy Solana programs to devnet
---

# Deploying PeridotVault to Solana Devnet

Follow these steps to deploy the `registry`, `pgc1`, and `game-store` programs to the Solana devnet.

## 1. Prepare Environment
Ensure your Solana CLI is configured for devnet.
```bash
solana config set --url devnet
```

Check your balance. You need at least 3-4 SOL for these three programs.
```bash
solana balance
```

If you need SOL:
```bash
solana airdrop 2
```
*(Note: You may need to run this multiple times or use a web faucet if the CLI airdrop is throttled.)*

## 2. Configure Anchor
Update the `provider` cluster in `Anchor.toml` to devnet.
```toml
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"
```

## 3. Verify Program IDs
Ensure the `[programs.devnet]` section in `Anchor.toml` matches the `declare_id!` in your Rust files:
- **Registry**: `DCYPxPtnVeBgy56SYMT6GPBMJp8NJNLmE46QfHYqCgGL`
- **PGC1**: `DzDbFZXZsmFFv1mMFimLaBjAQi7Z5gUaQ61qcDuR6Kor`
- **Game Store**: `6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd`

## 4. Build and Deploy
Run the build and then deploy specifically to devnet.
```bash
anchor build
anchor deploy --provider.cluster devnet
```

## 5. Verify on Explorer
After successful deployment, verify your program IDs on the [Solana Explorer (Devnet)](https://explorer.solana.com/?cluster=devnet).

## 6. Initialize (First Time Only)
Once deployed, you must run the initialization instructions using the console tool:
```bash
# In package.json, ensure "console" points to devnet if needed
node tests/console.cjs
```
Select `9. Init` to initialize the Registry and Game Store configs on devnet.

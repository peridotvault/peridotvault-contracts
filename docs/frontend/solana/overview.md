# Solana Overview

PeridotVault Solana is composed of three Anchor programs:

- PGL-1: game identity and license state
- Registry: publisher authorization, game registration, and payment-token policy
- Game Store: purchase flow, pricing, receipts, and store-level configuration

The programs are designed to be used together through controlled CPI paths. The SDK exposes IDLs, PDA helpers, deployment metadata, and documented builder conventions so application code can integrate without duplicating seed logic or program addresses.

## Deployment Environments

Use the deployment registry instead of hardcoding program IDs:

```ts
import { solana } from "@peridotvault/contracts";

const programs = solana.getSolanaPrograms("testnet");
```

The current repository explicitly defines `localnet` and `devnet` program IDs in `chains/solana/Anchor.toml`. The SDK maps `local` to `localnet` and `testnet` to the documented Devnet testing deployment. `staging` and `mainnet` are unresolved until the repo records their program IDs.

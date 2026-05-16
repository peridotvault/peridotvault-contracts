# Environments

PeridotVault uses named environments so application code does not hardcode program IDs.

## Supported Environments

- `local` — Resolved from `chains/solana/Anchor.toml` `[programs.localnet]`
- `staging` — Unresolved until a staging deployment is recorded
- `testnet` — Resolved from `chains/solana/Anchor.toml` `[programs.devnet]`
- `mainnet` — Unresolved until a mainnet deployment is recorded

## Solana Cluster Mapping

| Environment | Solana Cluster | Program IDs |
| --- | --- | --- |
| `local` | localnet | From Anchor.toml `[programs.localnet]` |
| `staging` | devnet / custom | Config file |
| `testnet` | devnet | From Anchor.toml `[programs.devnet]` |
| `mainnet` | mainnet-beta | Deployment registry |

## Usage

Use environment names in SDK helpers instead of raw public keys:

```ts
import { solana } from "@peridotvault/contracts";

const programs = solana.getSolanaPrograms("testnet");
// programs.registry — PublicKey on devnet
// programs.pgl1     — PublicKey on devnet
```

Unresolved Solana environments throw `PeridotContractsError` instead of returning placeholder production addresses.

## Deployment Lookup

The deployment helpers resolve program IDs across chains and environments:

<SandboxBlock template="deployments" />

## Next Steps

- [Installation](/guide/installation) — Setup and import patterns
- [Deployments](/guide/deployments) — Full deployment registry reference
- [Solana Overview](/solana/overview) — Solana program architecture

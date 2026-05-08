# Environments

PeridotVault uses named environments so application code does not hardcode program IDs.

Supported environment names:

- `local`
- `staging`
- `testnet`
- `mainnet`

Solana clusters map to environments:

| Environment | Solana status |
| --- | --- |
| `local` | Resolved from `chains/solana/Anchor.toml` `[programs.localnet]` |
| `staging` | Unresolved until a staging deployment is recorded |
| `testnet` | Resolved from `chains/solana/Anchor.toml` `[programs.devnet]` |
| `mainnet` | Unresolved until a mainnet deployment is recorded |

Use environment names in SDK helpers:

```ts
import { solana } from "@peridotvault/contracts";

const programs = solana.getSolanaPrograms("testnet");
```

Unresolved Solana environments throw `PeridotContractsError` instead of returning placeholder production addresses.

# SDK

The SDK exposes stable contract interface primitives for PeridotVault integrations.

## Exports

Primary entry points:

```ts
import { deployments, evm, solana } from "@peridotvault/contracts";
import { findLicensePda, getSolanaPrograms } from "@peridotvault/contracts/solana";
import { getEvmContractAddress } from "@peridotvault/contracts/evm";
import { getDeployment } from "@peridotvault/contracts/deployments";
```

## Solana Programs

Look up program IDs by environment name instead of hardcoding them:

<SandboxBlock template="solana-programs" />

## PDA Helpers

Derive deterministic program addresses from known seeds:

<SandboxBlock template="pda" />

## Current Surface

Current Solana SDK surface:

- IDL exports — `REGISTRY_IDL`, `PGL1_IDL`, `GAME_STORE_IDL`
- Type exports — Generated TypeScript types from IDLs
- Deployment helpers — `getSolanaPrograms()`, `getDeployment()`
- Program ID helpers — `getRegistryProgramId()`, `getPgl1ProgramId()`, `getGameStoreProgramId()`
- PDA helpers — `findLicensePda()`, `findRegistryConfigPda()`, and more

Planned higher-level surfaces:

- Instruction builders
- Transaction builders
- Account fetchers

Those higher-level helpers remain non-custodial and never sign or send transactions.

You can also explore the full generated API at the [API Reference](/sdk/api-reference).

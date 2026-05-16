# Introduction

`@peridotvault/contracts` is the official contract interface SDK for PeridotVault.

It is the shared source of truth for contract interfaces used by frontends, backend services, tests, indexers, and operational tooling.

The current documentation is Solana-first and covers:

- PGL-1 licensing
- Registry publishing and game registration
- Game Store purchase flows
- PDA derivation
- Deployment environments
- SDK helper conventions

The package is intentionally not a wallet SDK. It exposes deterministic interfaces and helper functions. Transaction signing and network submission should stay in the application layer.

## Try It Live

Edit the code below and see the SDK in action:

<SandboxBlock template="install" />

## Quick Start

```bash
pnpm add @peridotvault/contracts
```

```ts
import { deployments, evm, solana } from "@peridotvault/contracts";
import { findLicensePda } from "@peridotvault/contracts/solana";
import { getDeployment } from "@peridotvault/contracts/deployments";
```

## What's Inside

| Namespace | Subpath | Description |
| --- | --- | --- |
| Root | `@peridotvault/contracts` | Re-exports all namespaces |
| Solana | `@peridotvault/contracts/solana` | IDLs, PDA helpers, program ID helpers |
| EVM | `@peridotvault/contracts/evm` | ABI exports, contract address helpers |
| Deployments | `@peridotvault/contracts/deployments` | Multi-chain program ID lookup |

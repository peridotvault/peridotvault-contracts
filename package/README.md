# @peridotvault/contracts

`@peridotvault/contracts` is the official contract interface SDK for PeridotVault. It provides typed deployment metadata, Solana IDLs and PDA helpers, and EVM ABI/address helpers from a single npm package.

## Supported Networks

Supported chains:

- Solana
- Ethereum
- Base

Supported environments:

- `local`
- `staging`
- `testnet`
- `mainnet`

## Installation

```bash
pnpm add @peridotvault/contracts
```

## Root Import

```ts
import { deployments, evm, solana } from "@peridotvault/contracts";

const solanaPrograms = solana.getSolanaPrograms("testnet");
```

Solana program helpers return `PublicKey` values. Unresolved Solana environments, such as an undeployed mainnet, throw `PeridotContractsError` instead of returning placeholder production addresses.

## Solana PDA Helpers

```ts
import { PublicKey } from "@solana/web3.js";
import { findLicensePda } from "@peridotvault/contracts/solana";

const holder = new PublicKey("11111111111111111111111111111111");
const game = new PublicKey("11111111111111111111111111111111");

const [licensePda, bump] = findLicensePda(holder, game);
```

## Solana Registry Client

```ts
import { createRegistryClient } from "@peridotvault/contracts/solana";

const registry = createRegistryClient({
  provider,
  environment: "testnet"
});

const games = await registry.fetchGames();

await registry.updateGameStatus({
  registryGame,
  status: "suspended"
});
```

Every Registry write method also has a build-only `...Instruction` variant for dashboard previews.

## EVM Address Helpers

```ts
import { getEvmContractAddress } from "@peridotvault/contracts/evm";

const baseRegistry = getEvmContractAddress({
  chain: "base",
  environment: "testnet",
  contract: "registry"
});
```

## Deployment Helpers

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";

const deployment = getDeployment({
  chain: "solana",
  environment: "testnet"
});
```

## Package Layout

This repository publishes one npm package only: `@peridotvault/contracts`.

Use the root import for namespaced access:

```ts
import { solana, evm, deployments } from "@peridotvault/contracts";
```

Use subpath imports when you want a narrower surface:

```ts
import { findLicensePda } from "@peridotvault/contracts/solana";
import { getEvmDeployment } from "@peridotvault/contracts/evm";
import { getDeployment } from "@peridotvault/contracts/deployments";
```

The existing contract source remains under `chains/solana` and `chains/evm`. The SDK copies stable interface artifacts into `src/solana/idl`, `src/solana/types`, and `src/evm/abi` for packaging.

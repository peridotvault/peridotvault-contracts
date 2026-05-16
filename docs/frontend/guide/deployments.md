# Deployments

The deployment module provides typed, deterministic lookup of program IDs across all supported chains and environments.

## Getting a Deployment

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";

const dep = getDeployment({
  chain: "solana",
  environment: "testnet",
});
```

The returned deployment object includes:

```ts
{
  chain: "solana";
  environment: "testnet";
  programs: {
    registry: PublicKey;
    pgl1: PublicKey;
    gameStore?: PublicKey; // optional — not deployed on all envs
  };
}
```

## Try It

Change the `chain` and `environment` below to explore available deployments:

<SandboxBlock template="deployments" />

## Supported Chains

- `solana` — Devnet + (future) mainnet
- `ethereum` — Sepolia + (future) mainnet
- `base` — Base Sepolia + (future) mainnet

## Error Handling

Unresolved environments throw `PeridotContractsError`:

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";

try {
  const dep = getDeployment({ chain: "solana", environment: "mainnet" });
  // Throws — mainnet not yet deployed
} catch (err) {
  if (err instanceof PeridotContractsError) {
    console.error("Deployment not found:", err.message);
  }
}
```

## Static Deployment Exports

For direct access without the lookup helper:

```ts
import { SOLANA_DEPLOYMENTS, EVM_DEPLOYMENTS } from "@peridotvault/contracts/deployments";

console.log(SOLANA_DEPLOYMENTS.testnet.registry);  // PublicKey
console.log(EVM_DEPLOYMENTS.sepolia.registry);     // `0x...` address string
```

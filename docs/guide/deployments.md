# Deployments

The deployment registry normalizes program addresses by chain and environment.

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";

const deployment = getDeployment({
  chain: "solana",
  environment: "testnet"
});
```

Solana deployments include:

- PGL-1 program ID
- Registry program ID
- Game Store program ID
- Cluster name
- Optional program and interface versions

Unknown deployment addresses are represented with explicit TODO placeholder values until the corresponding environment is deployed.

Runtime helpers throw `PeridotContractsError` if an unresolved Solana environment or program is requested.

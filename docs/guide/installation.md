# Installation

Install the SDK from npm:

```bash
pnpm add @peridotvault/contracts
```

Root namespace import:

```ts
import { deployments, solana } from "@peridotvault/contracts";
```

Solana-only import:

```ts
import { findLicensePda } from "@peridotvault/contracts/solana";
```

Deployment-only import:

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";
```

The SDK is ESM-only.

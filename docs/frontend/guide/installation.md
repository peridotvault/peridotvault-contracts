# Installation

Install the SDK from npm:

```bash
pnpm add @peridotvault/contracts
```

## Import Patterns

### Root Namespace

Import the entire SDK surface from a single entry point:

```ts
import { deployments, solana } from "@peridotvault/contracts";
```

<SandboxBlock template="install" :height="350" />

### Solana-Only Import

Tree-shakeable subpath for Solana-specific helpers:

```ts
import { findLicensePda } from "@peridotvault/contracts/solana";
```

<SandboxBlock template="pda" :height="350" />

### Deployment-Only Import

Look up program IDs without pulling in the full SDK:

```ts
import { getDeployment } from "@peridotvault/contracts/deployments";
```

<SandboxBlock template="deployments" :height="350" />

## Requirements

- **ESM only** — No CommonJS fallback
- **TypeScript 5.x** — Types are bundled
- **Node.js 18+** or any modern bundler (Vite, Webpack 5, esbuild)
- **`@solana/web3.js`** peer dependency for Solana subpath

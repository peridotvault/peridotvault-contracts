---
layout: home
---

<div class="custom-home">
  <div class="hero">
    <h1>PeridotVault <span class="accent">Contracts</span></h1>
    <p>
      Official contract interface SDK for on-chain game licensing, publishing, and distribution.
      Solana-first. Multi-chain ready.
    </p>
    <div style="display:flex;gap:10px;margin-top:20px;">
      <a href="/guide/introduction" class="btn-brand">Get Started</a>
      <a href="https://github.com/peridotvault" class="btn-alt">View on GitHub</a>
    </div>
  </div>

  <div class="stat-grid">
    <div class="stat-card">
      <div class="stat-icon">&#9678;</div>
      <div class="stat-label">Chains</div>
      <div class="stat-value">3</div>
      <div class="stat-desc">Solana, Ethereum, Base</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">&#9636;</div>
      <div class="stat-label">Programs</div>
      <div class="stat-value">3</div>
      <div class="stat-desc">PGL-1, Registry, Game Store</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">&#8986;</div>
      <div class="stat-label">Environments</div>
      <div class="stat-value">4</div>
      <div class="stat-desc">Local, Staging, Testnet, Mainnet</div>
    </div>
    <div class="stat-card">
      <div class="stat-icon">&#9881;</div>
      <div class="stat-label">SDK Version</div>
      <div class="stat-value">0.1.0</div>
      <div class="stat-desc">ESM-only, TypeScript, tree-shakeable</div>
    </div>
  </div>

  <h2 style="font-size:18px;font-weight:600;margin-bottom:14px;color:var(--pvd-text-primary);">Explore</h2>

  <div class="feature-grid">
    <a class="feature-card" href="/guide/introduction">
      <span class="card-arrow">&rarr;</span>
      <h3>Introduction</h3>
      <p>Overview, architecture, and design principles of the PeridotVault contract SDK.</p>
    </a>
    <a class="feature-card" href="/guide/installation">
      <span class="card-arrow">&rarr;</span>
      <h3>Installation</h3>
      <p>Install the SDK, explore import patterns, and run live examples in your browser.</p>
    </a>
    <a class="feature-card" href="/solana/overview">
      <span class="card-arrow">&rarr;</span>
      <h3>Solana Programs</h3>
      <p>PGL-1 licensing, Registry publishing, Game Store purchases, PDA derivation.</p>
    </a>
    <a class="feature-card" href="/solana/registry-playground">
      <span class="card-arrow">&rarr;</span>
      <h3>Registry Playground</h3>
      <p>Connect your wallet and execute every Registry method interactively.</p>
    </a>
    <a class="feature-card" href="/sdk/">
      <span class="card-arrow">&rarr;</span>
      <h3>SDK Reference</h3>
      <p>Full API surface — deployment helpers, PDA functions, and interactive sandboxes.</p>
    </a>
    <a class="feature-card" href="/audit/">
      <span class="card-arrow">&rarr;</span>
      <h3>Audit Reports</h3>
      <p>Security findings, economic analysis, and remediation plans for all programs.</p>
    </a>
  </div>

  <div class="quick-start">
    <h2>Quick Start</h2>

```bash
pnpm add @peridotvault/contracts
```

```ts
import { deployments, evm, solana } from "@peridotvault/contracts";
import { findLicensePda } from "@peridotvault/contracts/solana";
import { getDeployment } from "@peridotvault/contracts/deployments";

const programs = solana.getSolanaPrograms("testnet");
const [pda, bump] = findLicensePda(holder, game);
const dep = getDeployment({ chain: "solana", environment: "testnet" });
```
  </div>

  <h2 style="font-size:18px;font-weight:600;margin-bottom:14px;color:var(--pvd-text-primary);">Ecosystem</h2>

  <div class="feature-grid">
    <a class="feature-card" href="/guide/environments">
      <span class="card-arrow">&rarr;</span>
      <h3>Environments</h3>
      <p>Named environments — local, staging, testnet, mainnet. No hardcoded addresses.</p>
    </a>
    <a class="feature-card" href="/guide/deployments">
      <span class="card-arrow">&rarr;</span>
      <h3>Deployments</h3>
      <p>Per-environment program IDs and contract addresses. Versioned, auditable, deterministic.</p>
    </a>
    <a class="feature-card" href="/solana/pda">
      <span class="card-arrow">&rarr;</span>
      <h3>PDA Reference</h3>
      <p>Complete reference of all program-derived addresses and their derivation seeds.</p>
    </a>
    <a class="feature-card" href="/solana/instructions">
      <span class="card-arrow">&rarr;</span>
      <h3>Instruction Builders</h3>
      <p>Build transaction instructions for all programs. Non-custodial, no signing.</p>
    </a>
    <a class="feature-card" href="/solana/transactions">
      <span class="card-arrow">&rarr;</span>
      <h3>Transaction Builders</h3>
      <p>Compose multi-instruction transactions with proper account ordering.</p>
    </a>
    <a class="feature-card" href="/solana/account-fetchers">
      <span class="card-arrow">&rarr;</span>
      <h3>Account Fetchers</h3>
      <p>Deserialize on-chain account data into typed TypeScript objects.</p>
    </a>
  </div>
</div>

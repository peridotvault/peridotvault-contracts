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

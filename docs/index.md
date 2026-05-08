# PeridotVault Contracts

`@peridotvault/contracts` is the official PeridotVault contract interface SDK.

The documentation is currently focused on Solana. It covers the PGL-1, Registry, and Game Store programs, plus the SDK interfaces used by applications and backend services that integrate with those programs.

The SDK provides:

- Solana IDLs and generated TypeScript IDL types
- Deployment registry helpers for supported environments
- PDA helpers for program-derived addresses
- Instruction builder documentation
- Transaction builder documentation
- Account fetcher documentation

Transaction builders prepare transaction objects for callers, but they do not sign or send transactions. Wallet approval, signing, submission, confirmation, and retry policy remain the responsibility of the consuming app.

## Start Here

- [Introduction](/guide/introduction)
- [Installation](/guide/installation)
- [Solana Overview](/solana/overview)
- [SDK Reference](/sdk/)
- [Audit Methodology](/audit/)

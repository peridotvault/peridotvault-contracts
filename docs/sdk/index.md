# SDK

The SDK exposes stable contract interface primitives for PeridotVault integrations.

Primary exports:

- `@peridotvault/contracts`
- `@peridotvault/contracts/solana`
- `@peridotvault/contracts/deployments`

Current Solana SDK surface:

- IDL exports
- Type exports
- Deployment helpers
- Program ID helpers
- PDA helpers

Planned higher-level surfaces:

- Instruction builders
- Transaction builders
- Account fetchers

Those higher-level helpers should remain non-custodial and should never sign or send transactions.

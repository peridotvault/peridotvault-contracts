# Deploy Solana Programs with Quasar

This workspace uses Quasar.

## Prerequisites

```bash
cargo install quasar-cli
solana --version
```

Set the target cluster with the Solana CLI:

```bash
solana config set --url localhost
# or
solana config set --url devnet
```

## Build

```bash
cd chains/solana
quasar build
```

For a Rust compile check without producing deployable SBF artifacts:

```bash
cargo check
```

## Deploy

Use Quasar/Solana deploy tooling for the generated program artifacts. Program IDs are declared in each program's `declare_id!`:

- `programs/pgl1/src/lib.rs`
- `programs/registry/src/lib.rs`
- `programs/game-store/src/lib.rs`

## Notes

- Legacy framework config and generated IDLs are intentionally removed.
- TypeScript clients/tests need Quasar-compatible builders before they can be reintroduced.
- Generated artifacts under `target/` are build outputs and should not be committed.

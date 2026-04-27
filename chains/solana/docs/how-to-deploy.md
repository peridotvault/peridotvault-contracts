# Deploy Solana Programs with Quasar

This workspace uses Quasar programs inside `programs/*`.

Quasar CLI v0.0.0 is single-program oriented. Its `quasar build` command does not accept a program path or workspace list, so this repository provides a root helper script for multi-program builds.

## Program IDs

The checked-in deploy keypairs match the `declare_id!` values:

| Program | Program ID | Keypair | Binary |
| --- | --- | --- | --- |
| `pgl1` | `5YctJfQJ6qfYDchYKyHFyjeKa3dx8Z6kg5pt68yaZ6c3` | `target/deploy/pgl1-keypair.json` | `target/deploy/pgl1.so` |
| `registry` | `8pgmtQDVpMX4FHmoCmWJCoB94RY56GKWUzo8f8e1Xfpo` | `target/deploy/registry-keypair.json` | `target/deploy/registry.so` |
| `game-store` | `8xi62uARkmBcKKwG3M8uvFnaayZL4MFvkQ91WG16eBCj` | `target/deploy/peridotvault_store-keypair.json` | `target/deploy/peridotvault_store.so` |

## Build All Programs

From the Solana workspace root:

```bash
cd chains/solana
./scripts/quasar-build-all.sh
```

The script:

- runs `quasar build` for `pgl1`, `registry`, and `game-store`
- regenerates consolidated IDLs in `target/idl/`
- regenerates TypeScript and Rust clients in `target/client/`
- removes per-program generated `programs/*/target/` directories

Generated outputs:

```text
target/deploy/pgl1.so
target/deploy/registry.so
target/deploy/peridotvault_store.so

target/idl/
target/client/typescript/
target/client/rust/
```

## Raw Per-Program Build

The helper above wraps these commands:

```bash
cd chains/solana/programs/pgl1
quasar build

cd ../registry
quasar build

cd ../game-store
quasar build
```

Running raw `quasar build` this way creates temporary per-program `target/idl` and `target/client` directories. Prefer `./scripts/quasar-build-all.sh` if you want a clean root-level `target`.

A workspace Rust check is still useful, but it does not create deployable SBF binaries:

```bash
cd chains/solana
cargo check
```

## Clean Generated Targets

Clean generated Quasar artifacts while preserving program keypairs:

```bash
cd chains/solana
./scripts/quasar-clean-targets.sh
```

The script preserves `target/deploy/*-keypair.json` because those files define the existing program IDs. It removes generated `.so`, IDL/client output, and per-program `target` directories.

## Localnet

Start a validator in one terminal:

```bash
solana-test-validator --reset
```

In another terminal:

```bash
cd chains/solana
solana config set --url http://127.0.0.1:8899
solana airdrop 10

solana program deploy target/deploy/pgl1.so \
  --program-id target/deploy/pgl1-keypair.json \
  --url http://127.0.0.1:8899

solana program deploy target/deploy/registry.so \
  --program-id target/deploy/registry-keypair.json \
  --url http://127.0.0.1:8899

solana program deploy target/deploy/peridotvault_store.so \
  --program-id target/deploy/peridotvault_store-keypair.json \
  --url http://127.0.0.1:8899
```

## Devnet

```bash
cd chains/solana
solana config set --url devnet
solana airdrop 2

solana program deploy target/deploy/pgl1.so \
  --program-id target/deploy/pgl1-keypair.json \
  --url devnet

solana program deploy target/deploy/registry.so \
  --program-id target/deploy/registry-keypair.json \
  --url devnet

solana program deploy target/deploy/peridotvault_store.so \
  --program-id target/deploy/peridotvault_store-keypair.json \
  --url devnet
```

## Mainnet

Use an explicitly funded payer and upgrade authority. Do not rely on implicit defaults.

```bash
cd chains/solana
solana config set --url mainnet-beta

solana program deploy target/deploy/pgl1.so \
  --program-id target/deploy/pgl1-keypair.json \
  --keypair ~/.config/solana/mainnet-payer.json \
  --upgrade-authority ~/.config/solana/mainnet-upgrade-authority.json \
  --url mainnet-beta

solana program deploy target/deploy/registry.so \
  --program-id target/deploy/registry-keypair.json \
  --keypair ~/.config/solana/mainnet-payer.json \
  --upgrade-authority ~/.config/solana/mainnet-upgrade-authority.json \
  --url mainnet-beta

solana program deploy target/deploy/peridotvault_store.so \
  --program-id target/deploy/peridotvault_store-keypair.json \
  --keypair ~/.config/solana/mainnet-payer.json \
  --upgrade-authority ~/.config/solana/mainnet-upgrade-authority.json \
  --url mainnet-beta
```

## Quasar Deploy Alternative

You can also use `quasar deploy` per program directory:

```bash
cd chains/solana/programs/pgl1
quasar deploy --url devnet --program-keypair ../../target/deploy/pgl1-keypair.json

cd ../registry
quasar deploy --url devnet --program-keypair ../../target/deploy/registry-keypair.json

cd ../game-store
quasar deploy --url devnet --program-keypair ../../target/deploy/peridotvault_store-keypair.json
```

For this workspace, direct `solana program deploy` from `chains/solana` is clearer because all three artifacts are in the shared workspace `target/deploy` directory.

## Troubleshooting: edition2024 / wincode build failure

If `quasar build` fails with:

```text
feature `edition2024` is required
wincode requires Cargo feature edition2024
```

then Cargo resolved newer transitive Solana crates than the Solana SBF toolchain can compile. `cargo check` may still pass because it uses your host Rust toolchain; `quasar build` uses the Solana SBF toolchain.

This repository pins compatible transitive versions in `Cargo.lock`:

```text
solana-address = 2.2.0
solana-instruction-view = 2.0.0
wincode = 0.4.9
wincode-derive = 0.4.3
```

If those versions are accidentally upgraded, restore the pins:

```bash
cd chains/solana
cargo update -p solana-address --precise 2.2.0
cargo update -p solana-instruction-view --precise 2.0.0
cargo update -p wincode-derive --precise 0.4.3
```

Then rebuild all programs:

```bash
./scripts/quasar-build-all.sh
```

If future Quasar/Solana crate versions require newer Rust even after those pins, update the Solana/Agave CLI. Check the SBF toolchain version with:

```bash
solana --version
cargo build-sbf --version
```

Update, then retry the per-program `quasar build` commands:

```bash
agave-install update
hash -r
solana --version
cargo build-sbf --version
```

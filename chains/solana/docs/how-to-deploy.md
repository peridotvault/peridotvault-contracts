# PeridotVault Solana — How to Deploy

## Program IDs (Current Devnet)

| Program        | ID                                             |
| -------------- | ---------------------------------------------- |
| **pgl1**       | `GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP` |
| **registry**   | `G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE` |
| **game-store** | `5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g` |

> **IMPORTANT:** Keypair files are saved in `keys/` directory. NEVER run `cargo clean` without backing up `target/deploy/*.json` first. If keypairs are lost, programs cannot be upgraded at the same address.

## Prerequisites

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor --tag v0.32.1 anchor-cli --locked

# Install Node dependencies
cd chains/solana
pnpm install
```

---

## Deploy ke Devnet

### 1. Setup Wallet & Network

```bash
solana config set --url https://api.devnet.solana.com
solana address
solana balance
solana airdrop 2   # jika perlu
```

### 2. Restore Keypairs (jika keypairs ada di keys/)

```bash
cp keys/*.json target/deploy/
```

### 3. Build & Deploy

```bash
pnpm anchor build

solana program deploy --url devnet target/deploy/pgl1.so
solana program deploy --url devnet target/deploy/registry.so
solana program deploy --url devnet target/deploy/game_store.so
```

### 4. Configure Programs

```bash
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/config.ts
```

### 5. Add Payment Tokens

```bash
# Registry accepted tokens (for registration fee)
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts registry-add <MINT> <FEE>

# Store accepted tokens (for buying games)
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set.ts store-add <MINT>
```

### 6. Verify

```bash
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/get.ts
```

---

## Deploy ke Mainnet

### 1. Setup Wallet

```bash
solana config set --url https://api.mainnet-beta.solana.com
solana balance   # perlu ~7 SOL untuk 3 program
```

### 2. Build

```bash
pnpm anchor build
```

### 3. Deploy

```bash
solana program deploy --url mainnet-beta target/deploy/pgl1.so
solana program deploy --url mainnet-beta target/deploy/registry.so
solana program deploy --url mainnet-beta target/deploy/game_store.so
```

### 4. Configure

```bash
ANCHOR_PROVIDER_URL=https://api.mainnet-beta.solana.com npx ts-node scripts/config.ts
```

---

## Generate New Program IDs

Jika perlu Program ID baru untuk deployment terpisah:

```bash
# 1. Backup keypair lama
cp -r keys/ keys-backup/

# 2. Generate baru
rm -f target/deploy/*.json
anchor build

# 3. Simpan keypair baru
cp target/deploy/*.json keys/

# 4. Ambil Program ID baru
solana-keygen pubkey target/deploy/game_store-keypair.json
solana-keygen pubkey target/deploy/pgl1-keypair.json
solana-keygen pubkey target/deploy/registry-keypair.json

# 5. Update semua reference
# - programs/*/src/lib.rs (declare_id!)
# - Anchor.toml
# - scripts/config.ts
# - docs/

# 6. Rebuild & deploy
pnpm anchor build
solana program deploy --url devnet target/deploy/*.so
```

---

## Upgrade Program

```bash
pnpm anchor build
solana program deploy --url devnet target/deploy/<program>.so
```

---

## Troubleshooting

### Error: `DeclaredProgramIdMismatch`

Penyebab: declare_id! di lib.rs tidak match dengan keypair di target/deploy/.

Solusi: Update declare_id! di lib.rs sesuai dengan `solana-keygen pubkey target/deploy/<name>-keypair.json`.

### Error: `insufficient funds`

Solusi: Airdrop lebih banyak SOL atau close old programs:

```bash
solana program close --url devnet --bypass-warning <OLD_PROGRAM_ID>
```

### Keypairs hilang setelah cargo clean

Solusi: Restore dari backup:

```bash
cp keys/*.json target/deploy/
anchor keys sync
```

---

## Estimasi Biaya

| Network  | Per Program | 3 Programs |
| -------- | ----------- | ---------- |
| Devnet   | ~1.5 SOL    | ~4.5 SOL   |
| Mainnet  | ~1.5 SOL    | ~4.5 SOL   |

Biaya = rent-exempt deposit. Bisa di-reclaim via `solana program close`.

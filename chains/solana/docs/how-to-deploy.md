# PeridotVault Solana — How to Deploy

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

## Program IDs

| Program        | ID                                             |
| -------------- | ---------------------------------------------- |
| **game-store** | `FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC` |
| **pgl1**       | `AHpAEMxUEk4Um3E6PgXxFQiiTBhSQP9Ej2Sy77Y7WU6H` |
| **registry**   | `2H2RfFxMYxh6njAJNekPacK671DL9q2W89YjiQhAM4ut` |

> **IMPORTANT:** Jika perlu generate Program ID baru, ikuti langkah di bagian [Generate New Program IDs](#generate-new-program-ids).

---

## Deploy ke Localnet

### 1. Clean & Build

```bash
cd chains/solana

# Hapus build artifact lama (penting!)
rm -rf target/deploy/*.json

# Build semua program
anchor build
```

### 2. Start Local Validator

```bash
# Stop validator lama jika ada
pkill -f solana-test-validator

# Hapus ledger lama
rm -rf test-ledger

# Start validator baru
solana-test-validator
```

### 3. Deploy

```bash
# Di terminal baru
cd chains/solana
anchor deploy --provider.cluster localnet
```

### 4. Verify

```bash
# Cek semua program on-chain
solana account FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC --url http://127.0.0.1:8899
solana account AHpAEMxUEk4Um3E6PgXxFQiiTBhSQP9Ej2Sy77Y7WU6H --url http://127.0.0.1:8899
solana account 2H2RfFxMYxh6njAJNekPacK671DL9q2W89YjiQhAM4ut --url http://127.0.0.1:8899

# Cek IDL accounts
anchor idl fetch FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC --provider.cluster localnet
```

---

## Deploy ke Devnet

### 1. Setup Wallet & Network

```bash
# Set cluster ke devnet
solana config set --url https://api.devnet.solana.com

# Cek wallet
solana address

# Cek balance
solana balance

# Airdrop jika balance < 2 SOL
solana airdrop 2
```

### 2. Clean & Build

```bash
cd chains/solana
rm -rf target/deploy/*.json
anchor keys sync
anchor build
```

### 3. Deploy

```bash
anchor deploy --provider.cluster devnet
```

### 4. Verify

```bash
solana account FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC --url devnet
solana account AHpAEMxUEk4Um3E6PgXxFQiiTBhSQP9Ej2Sy77Y7WU6H --url devnet
solana account 2H2RfFxMYxh6njAJNekPacK671DL9q2W89YjiQhAM4ut --url devnet
```

---

## Deploy ke Mainnet

### 1. Setup Wallet & Network

```bash
# Set cluster ke mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Pastikan wallet punya cukup SOL (estimasi 2-3 SOL untuk 3 program)
solana balance
```

### 2. Clean & Build

```bash
cd chains/solana
rm -rf target/deploy/*.json
anchor keys sync
anchor build
```

### 3. Deploy

```bash
anchor deploy --provider.cluster mainnet
```

> **Note:** Upgrade authority adalah wallet di `~/.config/solana/id.json`. Simpan keypair ini dengan aman — tanpa ini tidak bisa upgrade program.

### 4. Verify

```bash
solana account FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC --url mainnet
solana account AHpAEMxUEk4Um3E6PgXxFQiiTBhSQP9Ej2Sy77Y7WU6H --url mainnet
solana account 2H2RfFxMYxh6njAJNekPacK671DL9q2W89YjiQhAM4ut --url mainnet
```

---

## Generate New Program IDs

Jika perlu Program ID baru (misal untuk deployment berbeda):

### 1. Hapus keypair lama

```bash
cd chains/solana
rm -f target/deploy/*.json
```

### 2. Build (akan generate keypair baru)

```bash
anchor build
```

### 3. Ambil Program ID yang di-generate

```bash
solana-keygen pubkey target/deploy/game_store-keypair.json
solana-keygen pubkey target/deploy/pgl1-keypair.json
solana-keygen pubkey target/deploy/registry-keypair.json
```

### 4. Update semua file yang reference Program ID

Update Program ID di file-file berikut:

| File                             | Field                                                        |
| -------------------------------- | ------------------------------------------------------------ |
| `programs/game-store/src/lib.rs` | `declare_id!("...")`                                         |
| `programs/pgl1/src/lib.rs`       | `declare_id!("...")`                                         |
| `programs/registry/src/lib.rs`   | `declare_id!("...")`                                         |
| `Anchor.toml`                    | `[programs.localnet]` dan `[programs.devnet]`                |
| `tests/helpers/peridot.ts`       | `PGL1_PROGRAM_ID`, `REGISTRY_PROGRAM_ID`, `STORE_PROGRAM_ID` |
| `docs/game-store.md`             | Program ID di header                                         |

### 5. Rebuild & Deploy

```bash
rm -rf target/deploy/*.json
anchor build
anchor deploy --provider.cluster localnet
```

---

## Troubleshooting

### Error: `DeclaredProgramIdMismatch`

**Penyebab:** Program ID di `declare_id!` tidak match dengan keypair di `target/deploy/*.json`.

**Solusi:**

```bash
# 1. Hapus keypair lama
rm -f target/deploy/*.json

# 2. Build ulang (generate keypair baru)
anchor build

# 3. Ambil Program ID baru
solana-keygen pubkey target/deploy/game_store-keypair.json

# 4. Update declare_id! di lib.rs sesuai output di atas
# 5. Update Anchor.toml, tests/helpers/peridot.ts, docs
# 6. Deploy ulang
anchor deploy --provider.cluster localnet
```

### Error: `AccountNotFound` saat IDL fetch

**Penyebab:** IDL account belum dibuat atau deploy gagal di step IDL.

**Solusi:** Pastikan deploy berhasil sampai akhir (harus ada output `Idl account created: ...`).

### Error: `Connection refused`

**Penyebab:** Local validator tidak running.

**Solusi:**

```bash
pkill -f solana-test-validator
rm -rf test-ledger
solana-test-validator
```

---

## Estimasi Biaya Deploy

| Network  | Estimasi Biaya (3 program) |
| -------- | -------------------------- |
| Localnet | 0 SOL                      |
| Devnet   | ~2-3 SOL                   |
| Mainnet  | ~2-3.5 SOL                 |

Biaya = rent-exempt deposit untuk menyimpan program binary on-chain. Bukan gas fee — deposit ini bisa di-reclaim jika program di-close.

---

## Upgrade Program

```bash
# Build terbaru
anchor build

# Upgrade ke cluster target
anchor deploy --provider.cluster devnet   # atau mainnet
```

Upgrade authority harus sama dengan wallet yang deploy pertama kali.

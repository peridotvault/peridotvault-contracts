# PeridotVault Game Store v1

## Program Identity

| Field | Value |
|-------|-------|
| **Program ID** | `6gMd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd` |
| **Anchor module** | `peridotvault_store` |
| **Crate/lib** | `peridotvault-store` / `peridotvault_store` |

## Current Integration Status

- **Status:** Active integration (settlement + license mint sudah hidup)
- `buy_game` saat ini sudah menjalankan:
  - Validasi source/registry/token/listing
  - SPL token settlement (treasury, publisher, optional referrer)
  - CPI mint license ke PGL-1
  - Pembuatan purchase receipt + event

## Overview

Game Store adalah commerce layer untuk game canonical dari PGL-1 + Registry.

**Tanggung jawab utama:**
- Config global store (treasury, fee, referral, store_actor)
- Allowlist source program, registry program, payment token
- Listing config per game + payment option
- Discount/referral override per game
- Buy flow settlement + license issue + receipt

## Program Scope

### In Scope
- Listing configuration per game/token
- Pricing, discount, referral split, and payment settlement
- Receipt pencatatan pembelian
- Bridging purchase ke license issuance via CPI ke PGL-1

### Out of Scope
- Canonical game metadata ownership (tetap di PGL-1)
- Governance status game ecosystem (tetap di Registry)
- Revocation/refund/dispute final policy

## Authority Model

| Role | Description |
|------|-------------|
| **Store Admin** (`store_config.authority`) | Mengelola global config, program allowlist, token allowlist |
| **Publisher** (`pgl1::Game.publisher`) | Mengelola listing game miliknya (active/payment option/discount/referral) |
| **Registry** (`authorized_program` role=1) | Bisa call `init_game_store_config` dan `set_game_payment_option` tanpa publisher signature |
| **Buyer** | Melakukan purchase sesuai policy listing aktif |
| **Store Actor** (`store_config.store_actor`) | Actor operasional untuk CPI `mint_license` ke PGL-1, wajib authorized di PGL-1 |

## Program-Level Access Control

- **Admin gate:** Admin/config instruction wajib `has_one authority`
- **Publisher gate:** Mutasi listing wajib signer == `pgl1::Game.publisher`, tidak ada publisher pubkey yang disimpan di state store
- **Program allowlist gate:** External program harus ada dan `active=true` di allowlist store; `role=0` (Game Source) untuk referensi validasi game/license (PGL1, PGL2, dll); `role=1` (Registry) memiliki akses operasional — bisa call `init_game_store_config` dan `set_game_payment_option` tanpa publisher signature
- **Token allowlist gate:** Payment mint harus ada di `AcceptedPaymentToken` dan `active=true`
- **Buy gate:**
  - Registry status wajib Active
  - Listing/store config wajib active
  - Paid amount wajib match computed final price
  - Token account ownership/mint consistency wajib valid

## Authorized Actor Model

- **Internal authorization (store):** `AuthorizedProgram` mengontrol program external yang dipercaya dengan role-based access:
  - `role=0` (Game Source): referensi read-only untuk validasi game/license
  - `role=1` (Registry): akses operasional untuk setup game listing tanpa publisher signature
- **External authorization (PGL-1):** `store_actor` harus memiliki account `pgl1::AuthorizedActor` aktif; tanpa status authorized actor aktif, `buy_game` tidak bisa mint license

## Publisher Authority Model

- Game Store tidak menyimpan publisher pubkey di account state store
- Semua mutasi oleh publisher selalu divalidasi langsung terhadap `pgl1::Game.publisher`

## State Accounts

### 1. StoreConfig

| Field | Type |
|-------|------|
| authority | `Pubkey` |
| treasury | `Pubkey` |
| platform_fee_bps | `u16` |
| default_referral_bps | `u16` |
| max_referral_bps | `u16` |
| store_actor | `Pubkey` |
| bump | `u8` |

### 2. AuthorizedProgram

| Field | Type | Description |
|-------|------|-------------|
| program_id | `Pubkey` | Address of the authorized external program |
| active | `bool` | Whether this program is currently active |
| role | `u8` | `0` = Game Source (PGL1/PGL2...), `1` = Registry (operational access) |
| bump | `u8` | PDA bump |

**Role Constants:**
| Constant | Value | Description |
|----------|-------|-------------|
| `ROLE_SOURCE` | `0` | Game source programs (PGL1, PGL2, etc.) — read-only reference |
| `ROLE_REGISTRY` | `1` | Registry program — can call `init_game_store_config` and `set_game_payment_option` |

### 3. AcceptedPaymentToken

| Field | Type |
|-------|------|
| mint | `Pubkey` |
| active | `bool` |
| bump | `u8` |

### 4. GameStoreConfig

| Field | Type |
|-------|------|
| game | `Pubkey` |
| active | `bool` |
| referral_bps | `Option<u16>` |
| discount_bps | `Option<u16>` |
| discount_starts_at | `Option<i64>` |
| discount_expires_at | `Option<i64>` |
| bump | `u8` |

### 5. GamePaymentOption

| Field | Type |
|-------|------|
| game | `Pubkey` |
| mint | `Pubkey` |
| base_price | `u64` |
| active | `bool` |
| bump | `u8` |

### 6. PurchaseReceipt

| Field | Type |
|-------|------|
| buyer | `Pubkey` |
| game | `Pubkey` |
| payment_mint | `Pubkey` |
| paid_amount | `u64` |
| final_price | `u64` |
| referral_bps_applied | `u16` |
| purchased_at | `i64` |
| bump | `u8` |

## Constants

| Constant | Value |
|----------|-------|
| `BPS_DENOMINATOR` | `10_000` |
| `PLATFORM_FEE_BPS_MAX` | `10_000` |
| `MAX_REFERRAL_BPS_HARD_CAP` | `5_000` |

## PDA Seeds

| Account | Seeds |
|---------|-------|
| `store_config` | `["store_config"]` |
| `authorized_program` | `["authorized_program", program_id]` |
| `accepted_payment_token` | `["accepted_payment_token", mint_pubkey]` |
| `game_store_config` | `["game_store_config", game_pubkey]` |
| `game_payment_option` | `["game_payment_option", game_pubkey, mint_pubkey]` |
| `purchase_receipt` | `["purchase_receipt", buyer_pubkey, game_pubkey]` |

## Instruction Security Matrix

| Instruction | Caller | Guard |
|-------------|--------|-------|
| `initialize_store` | Signer pertama (authority awal) | `treasury != default`, `store_actor != default`, BPS boundary valid |
| `set_treasury` | Admin Store | `has_one authority`, `treasury != default` |
| `set_platform_fee` | Admin Store | `has_one authority`, `platform_fee_bps <= MAX`, `(platform_fee_bps + max_referral_bps) <= 10_000` |
| `set_default_referral` | Admin Store | `has_one authority`, `default_referral_bps <= max_referral_bps` |
| `set_max_referral` | Admin Store | `has_one authority`, `max <= HARD_CAP`, `default <= max`, `(platform_fee + max) <= 10_000` |
| `add_authorized_program` | Admin Store | `has_one authority`, init PDA, `active = true`, `role` param (0=source, 1=registry) |
| `update_authorized_program` | Admin Store | `has_one authority`, update `active` flag, optional `role` update |
| `add_payment_token` | Admin Store | `has_one authority`, init PDA, `active = true` |
| `update_payment_token` | Admin Store | `has_one authority`, update active flag |
| `init_game_store_config` | Publisher **OR** Registry (role=1) | Source (role=0) + Registry (role=1) authorized active, registry_game<->game match, registry status Active. If Publisher: `game.publisher == signer`. If Registry: no publisher signer needed. |
| `set_game_store_active` | Publisher | Publisher owner, registry_game match, registry Active |
| `set_game_payment_option` | Publisher **OR** Registry (role=1) | Source (role=0) + Registry (role=1) authorized active, registry_game<->game match, registry Active, game_store_config active, accepted token active, `base_price > 0`. If Publisher: `game.publisher == signer`. If Registry: no publisher signer needed. |
| `remove_game_payment_option` | Publisher | Publisher owner, source (role=0) authorized active, mint match, close PDA |
| `set_discount` | Publisher | Publisher owner, source (role=0) authorized active, registry Active, `bps <= 10_000`, `start < end` |
| `clear_discount` | Publisher | Publisher owner, source (role=0) authorized active, reset discount fields ke None |
| `set_referral_bps` | Publisher | Publisher owner, source (role=0) authorized active, `value <= max_referral_bps`, normalisasi `Some(0) -> None` |
| `set_store_actor` | Admin Store | `has_one authority`, `new_store_actor != default` |
| `buy_game` | Buyer (+ store_actor sebagai signer terpisah) | Registry Active, listing aktif, `paid_amount == final_price`, token accounts konsisten, store_actor authorized di PGL-1, **license PDA harus kosong** (belum punya / sudah expired-burned), mint license via CPI + receipt write |

## Flow Per Instruction

### 1. `initialize_store(...)`
- Validasi treasury/store_actor non-default
- Validasi BPS invariants (`platform_fee <= MAX`, `max_referral <= HARD_CAP`, `default <= max`, `platform_fee + max <= 10_000`)
- Init store_config

### 2. `set_treasury`
- Validasi admin
- Validasi `treasury != default`
- Update treasury

### 3. `set_platform_fee`
- Validasi admin
- Validasi `platform_fee_bps <= PLATFORM_FEE_BPS_MAX`
- Validasi `(platform_fee_bps + max_referral_bps) <= 10_000`
- Update `platform_fee_bps`

### 4. `set_default_referral`
- Validasi admin
- Validasi `default_referral_bps <= max_referral_bps`
- Update `default_referral_bps`

### 5. `set_max_referral`
- Validasi admin
- Validasi `max <= MAX_REFERRAL_BPS_HARD_CAP`
- Validasi `default_referral_bps <= max`
- Validasi `(platform_fee_bps + max) <= 10_000`
- Update `max_referral_bps`

### 6. `add_authorized_program(role)`
- Validasi admin
- Validasi `role <= ROLE_REGISTRY`
- Init PDA dengan `active = true`, role sesuai param

### 7. `update_authorized_program(active, role?)`
- Validasi admin
- Update `active` flag
- Optional: update `role` (jika provided, validasi `role <= ROLE_REGISTRY`)

### 8. `add_payment_token`
- Validasi admin
- Init PDA dengan `active = true`

### 9. `update_payment_token`
- Validasi admin
- Update active flag

### 10. `init_game_store_config(active)`
- Validasi source (role=0) + registry (role=1) authorized active
- Validasi registry_game match + registry status Active
- Authorization: jika `publisher` signer provided → validasi `game.publisher == signer`; jika tidak → validasi registry role=1
- Init game_store_config (`referral_bps=None`, discount fields=None)

### 11. `set_game_store_active(active)`
- Validasi publisher ownership + linkage game/registry
- Validasi registry Active
- Update active flag

### 12. `set_game_payment_option(base_price, active)`
- Validasi source (role=0) + registry (role=1) authorized active
- Validasi registry active + game_store_config active
- Validasi accepted payment token active
- Validasi `base_price > 0`
- Authorization: jika `publisher` signer provided → validasi `game.publisher == signer`; jika tidak → validasi registry role=1
- `init_if_needed` dan update game_payment_option

### 13. `remove_game_payment_option()`
- Validasi publisher ownership + mint match
- Validasi source (role=0) authorized active
- Close game_payment_option (lamport refund ke publisher)

### 14. `set_discount(discount_bps, starts_at, expires_at)`
- Validasi publisher ownership + source (role=0) authorized active + registry Active
- Validasi `bps <= 10_000` dan `start < end`
- Update field discount

### 15. `clear_discount()`
- Validasi publisher ownership + source (role=0) authorized active
- Reset seluruh field discount ke None

### 16. `set_referral_bps(referral_bps)`
- Validasi publisher ownership + source (role=0) authorized active
- Normalisasi value:
  - `None` -> `None`
  - `Some(0)` -> `None`
  - `Some(v>0)` -> `Some(v)`, `v <= max_referral_bps`
- Update `referral_bps`

### 17. `set_store_actor(new_store_actor)`
- Validasi admin
- Validasi `new_store_actor != default`
- Update store_actor

### 18. `buy_game(paid_amount, referrer)`
- Validasi registry status, listing status, payment mint, dan final price
- **Validasi ownership:** license PDA wajib kosong (belum pernah beli ATAU license sudah expired/burned/closed)
- Hitung split nominal:
  - `platform_fee_amount`
  - `publisher_amount`
  - `referral_amount` (jika referrer ada)
- Transfer SPL token buyer -> treasury
- Transfer SPL token buyer -> publisher
- Jika `referral_amount > 0`, transfer buyer -> referrer (dengan validasi token account referrer)
- CPI `pgl1::mint_license` menggunakan `store_actor` yang authorized
- Simpan/update purchase_receipt dan emit event

## Buy Flow (Current vs Target)

### Current
- Settlement SPL token + mint license + receipt sudah aktif

### Target lanjutan (belum ada)
- Refund/cancel flow
- Dispute/reconciliation flow
- Anti-fraud policy layer tambahan

## Operational Prerequisites

1. `initialize_store` sudah dijalankan
2. `authorized_program` sudah aktif (source role=0 + registry role=1)
3. `accepted_payment_token` store sudah aktif
4. Game sudah terdaftar di Registry dengan status `Active`
5. `init_game_store_config` + `set_game_payment_option` sudah dilakukan publisher
6. `store_actor` ada di PGL-1 `authorized_actor` dan status active

## Events

| Event | Description |
|-------|-------------|
| `StoreInitialized` | Store berhasil diinisialisasi |
| `TreasuryUpdated` | Treasury address diubah |
| `PlatformFeeUpdated` | Platform fee BPS diubah |
| `DefaultReferralUpdated` | Default referral BPS diubah |
| `MaxReferralUpdated` | Max referral BPS diubah |
| `AuthorizedProgramAdded` | Program baru ditambahkan ke allowlist dengan role tertentu |
| `AuthorizedProgramUpdated` | Status active atau role program diubah |
| `PaymentTokenAdded` | Payment token baru ditambahkan ke allowlist |
| `PaymentTokenUpdated` | Status active payment token diubah |
| `GameStoreConfigInitialized` | Game store config baru diinisialisasi |
| `GameStoreActiveUpdated` | Status active game store diubah |
| `GamePaymentOptionSet` | Payment option untuk game diset |
| `GamePaymentOptionRemoved` | Payment option untuk game dihapus |
| `DiscountSet` | Discount untuk game diset |
| `DiscountCleared` | Discount untuk game direset |
| `ReferralBpsUpdated` | Referral BPS untuk game diubah |
| `StoreActorUpdated` | Store actor address diubah |
| `GamePurchased` | Game berhasil dibeli |
| `PurchaseReceiptCreated` | Purchase receipt berhasil dibuat |

## Errors

| Error | Description |
|-------|-------------|
| `Unauthorized` | Signer tidak memiliki akses |
| `InvalidPlatformFeeBps` | Platform fee BPS melebihi batas |
| `InvalidDefaultReferralBps` | Default referral BPS melebihi max |
| `InvalidMaxReferralBps` | Max referral BPS melebihi batas atau melanggar invariant |
| `ReferralAboveMax` | Referral BPS melebihi max_referral_bps |
| `SourceProgramNotAuthorized` | Source program (role=0) tidak ada atau tidak active di allowlist |
| `RegistryProgramNotAuthorized` | Registry program (role=1) tidak ada atau tidak active di allowlist |
| `PaymentTokenNotAllowed` | Payment mint tidak ada di allowlist |
| `PaymentTokenDisabled` | Payment token ada tapi status tidak active |
| `InvalidPrice` | Base price tidak valid (<= 0) |
| `PriceNotFound` | Payment option tidak aktif |
| `StoreGameInactive` | Game store config tidak active |
| `GameNotActive` | Game tidak active di registry |
| `GameNotRegistered` | Game tidak terdaftar |
| `AlreadyOwned` | Buyer sudah memiliki license untuk game ini |
| `InvalidDiscountBps` | Discount BPS melebihi 10_000 |
| `InvalidDiscountWindow` | Discount start >= end |
| `InvalidReferralBps` | Referral BPS + platform fee melebihi 10_000 |
| `MathOverflow` | Operasi aritmatika overflow |
| `InvalidPaymentAmount` | Paid amount tidak match final price atau <= 0 |
| `UnsupportedSourceGameOwner` | Owner game dari source program tidak didukung |
| `RegistryGameMismatch` | Game PDA tidak match dengan registry_game.game |
| `PaymentFailed` | Transfer SPL token gagal |
| `LicenseMintFailed` | CPI mint_license gagal atau license PDA tidak match |
| `MissingReferrerTokenAccount` | Referrer payment account tidak disediakan |
| `InvalidReferrerTokenAccount` | Referrer token account owner/mint tidak match |
| `InvalidTreasury` | Treasury address tidak valid |
| `InvalidStoreActor` | Store actor address tidak valid |
| `GamePaymentOptionMismatch` | Game di payment option tidak match dengan game yang dimaksud |
| `InvalidRole` | Role value tidak valid (melebihi ROLE_REGISTRY) |
| `InsufficientRole` | Program role tidak cukup untuk melakukan action ini |

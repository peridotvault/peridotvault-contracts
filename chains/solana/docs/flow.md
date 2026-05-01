# PeridotVault — Create Game Flow

> **Status:** Deployed on **Devnet** (testing). Not yet on mainnet (production).

## Flow Diagram

```
┌─────────────────────────────────────────┐
│          CREATE GAME + REGISTER          │
└─────────────────────────────────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │ 1. Pick Payment Method        │
    │    (Registry Accepted Token)   │
    │                               │
    │  If user is Grant Publisher:  │
    │    → Skip registration fee     │
    │    → Show "Free (Partner)"     │
    │  Else:                        │
    │    → Pay registration fee      │
    │    → Show "Fee: X tokens"     │
    └───────────────────────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │ 2. Game ID                    │
    │    e.g. "my-awesome-game"     │
    └───────────────────────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │ 3. Metadata URI               │
    │    e.g. "https://meta...json" │
    └───────────────────────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │ 4. Is this game free? (y/n)  │
    └───────┬───────────────────────┘
            │
     ┌──────┴──────┐
     │             │
     ▼ y (paid)    ▼ n (free)
┌─────────────┐  ┌──────────────────┐
│ 5a. Pick    │  │ 5b. Init store   │
│  Store      │  │  config only      │
│  Accepted   │  │ (buy/sell free    │
│  Token      │  │  game license)    │
│             │  └──────────────────┘
│ 5c. Set     │          │
│  Base Price │          │
│  (tokens)   │          │
└─────────────┘          │
       │                 │
       └────────┬────────┘
                │
                ▼
    ┌───────────────────────────────┐
    │ 6. Create Game + Register     │
    │    - CPI: PGL1.create_game    │
    │    - Init: RegistryGame       │
    │    - CPI: InitGameStoreConfig │
    │    - CPI: SetPaymentOption    │
    │      (only if paid)           │
    └───────────────────────────────┘
                │
                ▼
    ┌───────────────────────────────┐
    │ 7. Done!                      │
    │    - game PDA                 │
    │    - registry_game PDA        │
    │    - game_store_config PDA    │
    │    - game_payment_option PDA  │
    │      (only if paid)           │
    └───────────────────────────────┘
```

## States

### RegistryGame Status
- `Active` (0) — game is live, can be purchased
- `Suspended` (1) — temporarily disabled
- `Banned` (2) — permanently disabled

### Game Payment Option
- When `base_price = Some(price)` → paid game, payment option created
- When `base_price = None` → free game, only store config created
  (players can still trade license via store mechanisms)

## PDAs

| Account            | Seeds                                     | Program   |
|--------------------|-------------------------------------------|-----------|
| pgl_config         | `["pgl_config"]`                          | pgl1      |
| registry_config    | `["registry_config"]`                     | registry  |
| store_config       | `["store_config"]`                        | game-store|
| game               | `["game", creator, nonce_le]`             | pgl1      |
| creator_state      | `["creator_state", creator]`              | pgl1      |
| registry_game      | `["registry_game", game]`                 | registry  |
| publish_grant      | `["publish_grant", publisher]`           | registry  |
| game_store_config  | `["game_store_config", game]`             | game-store|
| game_payment_option| `["game_payment_option", game, mint]`     | game-store|

## Program IDs (Devnet)

| Program     | Address                                    |
|-------------|--------------------------------------------|
| PGL1        | GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP |
| Registry    | G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE |
| Game Store  | 5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g |

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
- A game can have **0, 1, or many** `GamePaymentOption` PDAs (one per accepted mint)
- Free game (0 payment options): buyers call `buy_game(mint_token = None)` — payment accounts are optional, license is minted directly

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

---

# PeridotVault — Buy Game Flow

## Flow Diagram

```
┌─────────────────────────────────────────┐
│               BUY GAME                  │
│  buyer calls buy_game(mint_token,       │
│        referrer?)                       │
└─────────────────────────────────────────┘
                    │
                    ▼
    ┌───────────────────────────────┐
    │ 1. Validate Common            │
    │    - registry_game == game    │
    │    - registry status Active   │
    │    - game_store_config active │
    └───────────────────────────────┘
                    │
                    ▼
            mint_token.is_some()?
                    │
         ┌──────────┴──────────┐
         │ yes (PAID)          │ no (FREE)
         ▼                     ▼
 ┌──────────────────────┐  ┌──────────────────────┐
 │ PAID PATH            │  │ FREE PATH            │
 │                      │  │                      │
 │ 2a. Validate payment │  │ 2b. Skip payment     │
 │  option exists &     │  │  accounts (all are   │
 │  active (manual PDA  │  │  Option::None)       │
 │  check: [game,mint]) │  │                      │
 │                      │  │  final_price = 0     │
 │ 2b. Validate accepted│  │  payment_mint =      │
 │  payment token       │  │    default           │
 │  exists & active     │  │  referral_bps = 0    │
 │  (manual PDA check)  │  └──────────┬───────────┘
 │                      │             │
 │ 2c. base_price →     │             │
 │  final_price         │             │
 │  (apply discount)    │             │
 │                      │             │
 │ 2d. Validate token   │             │
 │  accounts (buyer,    │             │
 │  publisher, treasury)│             │
 │                      │             │
 │ 3. Settlement        │             │
 │  (SPL transfers):    │             │
 │  - buyer → treasury  │             │
 │  - buyer → publisher │             │
 │  - buyer → referrer  │             │
 │    (if referral>0)   │             │
 └──────────┬───────────┘             │
            │                         │
            └──────────┬──────────────┘
                       │
                       ▼
    ┌───────────────────────────────┐
    │ 4. Mint License (CPI)         │
    │    - Derive license PDA:      │
    │      ["license", buyer, game] │
    │    - Validate PDA match       │
    │    - Validate license empty   │
    │      (not already owned)      │
    │    - CPI pgl1::mint_license   │
    │      via store_actor          │
    └───────────────────────────────┘
                       │
                       ▼
    ┌───────────────────────────────┐
    │ 5. Write PurchaseReceipt      │
    │    init_if_needed PDA:        │
    │    ["purchase_receipt",       │
    │     buyer, game]              │
    │                               │
    │    Fields:                    │
    │    - buyer, game              │
    │    - payment_mint             │
    │      (default for free)       │
    │    - paid_amount, final_price │
    │      (both 0 for free)        │
    │    - referrer, referral_bps   │
    │    - purchased_at             │
    └───────────────────────────────┘
                       │
                       ▼
    ┌───────────────────────────────┐
    │ 6. Emit Events                │
    │    - GamePurchased            │
    │    - PurchaseReceiptCreated   │
    └───────────────────────────────┘
                       │
                       ▼
    ┌───────────────────────────────┐
    │ DONE! Buyer owns license      │
    │ (License PDA on PGL-1)       │
    └───────────────────────────────┘
```

## Payment Accounts (per path)

| Account | Paid (`mint_token = Some`) | Free (`mint_token = None`) | Notes |
|---------|:-----------:|:------------:|-------|
| `payment_mint` | Must be `Some` | Must be `None` | SPL mint chosen by buyer |
| `accepted_payment_token` | Must be `Some`, active | Must be `None` | Manual PDA validation |
| `game_payment_option` | Must be `Some`, active | Must be `None` | Manual PDA validation; one per (game, mint) |
| `buyer_payment_account` | Must be `Some`, mut | Must be `None` | Buyer's token account |
| `publisher_payment_account` | Must be `Some`, mut | Must be `None` | Publisher's token account |
| `treasury_payment_account` | Must be `Some`, mut | Must be `None` | Treasury token account |
| `referrer_payment_account` | `Some` if referral>0 | Must be `None` | Referrer token account |

## Multi-Token Pricing

A single game can have multiple `GamePaymentOption` PDAs — one per accepted mint:

```
Game "epic-quest"
 ├── GamePaymentOption(game=epic-quest, mint=USDC, base_price=10)
 ├── GamePaymentOption(game=epic-quest, mint=SOL,  base_price=0.05)
 └── GamePaymentOption(game=epic-quest, mint=BONK, base_price=500000)
```

The buyer selects the token by passing the desired `payment_mint` account. The PDA
`["game_payment_option", game.key(), mint.key()]` resolves to the correct pricing.

## Settlement Split (Paid Only)

```
final_price
    ├── platform_fee    = final_price * platform_fee_bps / 10_000   → treasury
    ├── referral_amount = final_price * referral_bps / 10_000       → referrer
    └── publisher_amount = final_price - platform_fee - referral     → publisher
```

Constraints: `platform_fee_bps + referral_bps <= 10_000` (100%).

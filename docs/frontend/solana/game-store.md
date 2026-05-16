# Game Store Architecture

The Game Store program manages purchase-facing configuration and receipts.

Primary responsibilities:

- Initialize store configuration
- Manage authorized source and registry programs
- Manage accepted payment tokens
- Configure game store availability
- Configure game payment options
- Apply discounts and referral settings
- Create purchase receipts
- CPI into PGL-1 for license minting

Core accounts:

- `store_config`
- `authorized_program`
- `accepted_payment_token`
- `game_store_config`
- `game_payment_option`
- `purchase_receipt`

The Game Store should only trust source programs and registry programs recorded in its authorized-program accounts.

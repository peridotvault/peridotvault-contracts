# Game Store Audit

Review scope:

- Store configuration
- Authorized program accounts
- Accepted payment tokens
- Game store configuration
- Payment options
- Discounts and referrals
- Purchase receipts
- License mint CPI flow

Key checks:

- Store purchases require active game and payment configuration
- Payment math handles native and token payments correctly
- Discounts and referrals stay inside configured caps
- Purchase receipt PDA prevents duplicate purchase state
- CPI to PGL-1 cannot be redirected to an unauthorized program

Status: initial report placeholder.

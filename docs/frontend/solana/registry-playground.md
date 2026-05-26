# Registry Playground

Execute every Registry method directly from your browser. Connect a Solana wallet to call read methods (instant) and write methods (transaction signing).

<RegistryPlayground />

## Supported Wallets

- **Phantom** — Install from [phantom.app](https://phantom.app)
- **Solflare** — Install from [solflare.com](https://solflare.com)
- **Backpack** — Install from [backpack.app](https://backpack.app)

## Environments

| Environment | Cluster | Requires |
| --- | --- | --- |
| `local` | localnet | Local validator running on `localhost:8899` |
| `testnet` | devnet | Devnet SOL (faucet at [solfaucet.com](https://solfaucet.com)) |
| `staging` | devnet / custom | Deployed staging programs |
| `mainnet` | mainnet-beta | Real SOL |

## Methods Reference

### Registry Config

- **initializeRegistry** — Initialize the Registry with a treasury address. Call once.
- **setTreasury** — Update the treasury address.

### Payment Tokens

- **addPaymentToken** — Add an accepted payment token with a fee amount.
- **updatePaymentToken** — Enable/disable a token and update its fee.
- **removePaymentToken** — Remove an accepted payment token.

### Publish Grants

- **createPublishGrant** — Grant publish permission to an address.
- **updatePublishGrant** — Update a publish grant's expiration.

### Game Management

- **updateGameStatus** — Set a registered game as active, suspended, or banned.
- **closeRegistryGame** — Close a registry game account and reclaim rent.

### Queries

- **fetchGames** — Fetch all registered games (no signature needed).
- **fetchGame** — Fetch a single game's details by its registry account address.

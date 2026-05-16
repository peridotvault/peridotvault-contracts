# Registry

## Overview

The Registry program is PeridotVault's publishing and game-registration policy layer.

It manages:

- Registry authority and treasury
- Accepted payment tokens and publish fees
- Publisher publish grants
- Game registration into PGL-1
- Registry game status
- CPI setup into the Game Store

Frontend apps should use `createRegistryClient` from `@peridotvault/contracts/solana`. The client hides the IDL, program ID, and PDA details so your UI can call package functions directly.

## Setup

Install:

```bash
pnpm add @peridotvault/contracts @coral-xyz/anchor @solana/web3.js
```

Create the client:

```ts
import { AnchorProvider } from "@coral-xyz/anchor";
import { Connection } from "@solana/web3.js";
import { createRegistryClient } from "@peridotvault/contracts/solana";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const provider = new AnchorProvider(connection, wallet, {
  commitment: "confirmed",
});

const registry = createRegistryClient({
  provider,
  environment: "testnet",
});
```

`wallet` is the connected wallet from your Solana wallet adapter. It must satisfy Anchor's wallet shape: `publicKey`, `signTransaction`, and `signAllTransactions`.

## What The Client Resolves

The Registry client automatically resolves:

- Registry IDL
- PGL-1 IDL for `createGameAndRegister`
- Registry program ID
- PGL-1 program ID
- Game Store program ID
- Registry config PDA
- Registry payment token PDA
- Publish grant PDA
- Registry game PDA
- PGL-1 creator state PDA
- PGL-1 game PDA
- PGL-1 config PDA
- Game Store authorization PDAs
- Game Store config and payment option PDAs

Your frontend should pass business inputs such as `mint`, `gameId`, `metadataUri`, `status`, and `treasury`, not raw IDL/PDA wiring.

## Return Types

Write functions return transaction signatures:

```ts
type Signature = string;
```

Build-only functions return Solana instructions:

```ts
import type { TransactionInstruction } from "@solana/web3.js";
```

Read functions return decoded account data:

```ts
import type { BN } from "@coral-xyz/anchor";
import type { PublicKey } from "@solana/web3.js";

type RegistryGameRow = {
  publicKey: PublicKey;
  game: PublicKey;
  gameId: string;
  registeredAt: BN;
  status: "active" | "suspended" | "banned" | "unknown";
};
```

## Build vs Execute

Every Registry write function has two variants.

Build-only variant:

```ts
const ix = await registry.updateGameStatusInstruction({
  registryGame,
  status: "suspended",
});
```

This returns a `TransactionInstruction`. Use it for dashboard previews, custom transaction composition, priority fees, or multisig flows.

Execute variant:

```ts
const signature = await registry.updateGameStatus({
  registryGame,
  status: "suspended",
});
```

This sends through Anchor `.rpc()` using the provider wallet.

## Example

Fetch all games, display them in a table, and suspend one game:

```ts
const games = await registry.fetchGames();

const rows = games.map((game) => ({
  registryGame: game.publicKey.toBase58(),
  pglGame: game.game.toBase58(),
  gameId: game.gameId,
  registeredAt: game.registeredAt.toNumber(),
  status: game.status,
}));

await registry.updateGameStatus({
  registryGame: games[0].publicKey,
  status: "suspended",
});
```

## Functions

| Function                           | Type    | Purpose                            |
| ---------------------------------- | ------- | ---------------------------------- |
| `fetchGames`                       | read    | Fetch all Registry game accounts   |
| `fetchGame`                        | read    | Fetch one Registry game account    |
| `initializeRegistryInstruction`    | build   | Build `initialize_registry`        |
| `initializeRegistry`               | execute | Execute `initialize_registry`      |
| `setTreasuryInstruction`           | build   | Build `set_treasury`               |
| `setTreasury`                      | execute | Execute `set_treasury`             |
| `addPaymentTokenInstruction`       | build   | Build `add_payment_token`          |
| `addPaymentToken`                  | execute | Execute `add_payment_token`        |
| `updatePaymentTokenInstruction`    | build   | Build `update_payment_token`       |
| `updatePaymentToken`               | execute | Execute `update_payment_token`     |
| `removePaymentTokenInstruction`    | build   | Build `remove_payment_token`       |
| `removePaymentToken`               | execute | Execute `remove_payment_token`     |
| `createPublishGrantInstruction`    | build   | Build `create_publish_grant`       |
| `createPublishGrant`               | execute | Execute `create_publish_grant`     |
| `updatePublishGrantInstruction`    | build   | Build `update_publish_grant`       |
| `updatePublishGrant`               | execute | Execute `update_publish_grant`     |
| `createGameAndRegisterInstruction` | build   | Build `create_game_and_register`   |
| `createGameAndRegister`            | execute | Execute `create_game_and_register` |
| `updateGameStatusInstruction`      | build   | Build `update_game_status`         |
| `updateGameStatus`                 | execute | Execute `update_game_status`       |
| `closeRegistryGameInstruction`     | build   | Build `close_registry_game`        |
| `closeRegistryGame`                | execute | Execute `close_registry_game`      |

## fetchGames

Fetches all `registryGame` accounts.

Signature:

```ts
fetchGames(): Promise<RegistryGameRow[]>
```

Parameters: none.

Returns:

| Field          | Type                                               | Meaning                         |
| -------------- | -------------------------------------------------- | ------------------------------- |
| `publicKey`    | `PublicKey`                                        | Registry game account address   |
| `game`         | `PublicKey`                                        | PGL-1 game account address      |
| `gameId`       | `string`                                           | Human-readable game id          |
| `registeredAt` | `BN`                                               | Unix timestamp from the program |
| `status`       | `"active" \| "suspended" \| "banned" \| "unknown"` | Registry status                 |

Example:

```ts
const games = await registry.fetchGames();
```

## fetchGame

Fetches one `registryGame` account.

Signature:

```ts
fetchGame(registryGame: PublicKey): Promise<RegistryGameAccount>
```

Parameters:

| Name           | Type        | Required | Description       |
| -------------- | ----------- | -------- | ----------------- |
| `registryGame` | `PublicKey` | Yes      | Registry game PDA |

Example:

```ts
const account = await registry.fetchGame(registryGame);
```

## initializeRegistry

Creates the singleton Registry config account. This is an admin/bootstrap instruction.

Execute signature:

```ts
initializeRegistry(params: {
  treasury: PublicKey;
}): Promise<string>
```

Build signature:

```ts
initializeRegistryInstruction(params: {
  treasury: PublicKey;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type        | Required   | Description                                         |
| ----------- | ----------- | ---------- | --------------------------------------------------- |
| `treasury`  | `PublicKey` | Yes        | Treasury wallet that receives Registry funds/rent   |
| `authority` | `PublicKey` | Build only | Optional authority override for instruction preview |

Signer:

- Execute: provider wallet signs as `authority`.
- Build: returned instruction marks `authority` as signer.

Example:

```ts
const signature = await registry.initializeRegistry({
  treasury,
});
```

## setTreasury

Updates the Registry treasury. The connected wallet must be the Registry authority.

Execute signature:

```ts
setTreasury(params: {
  treasury: PublicKey;
}): Promise<string>
```

Build signature:

```ts
setTreasuryInstruction(params: {
  treasury: PublicKey;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type        | Required   | Description                 |
| ----------- | ----------- | ---------- | --------------------------- |
| `treasury`  | `PublicKey` | Yes        | New treasury wallet         |
| `authority` | `PublicKey` | Build only | Optional authority override |

Example:

```ts
await registry.setTreasury({
  treasury: newTreasury,
});
```

## addPaymentToken

Adds an accepted payment mint and sets its publish fee.

Execute signature:

```ts
addPaymentToken(params: {
  mint: PublicKey;
  feeAmount: number | bigint | string | BN;
}): Promise<string>
```

Build signature:

```ts
addPaymentTokenInstruction(params: {
  mint: PublicKey;
  feeAmount: number | bigint | string | BN;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type                               | Required   | Description                              |
| ----------- | ---------------------------------- | ---------- | ---------------------------------------- |
| `mint`      | `PublicKey`                        | Yes        | SPL token mint accepted for publish fees |
| `feeAmount` | `number \| bigint \| string \| BN` | Yes        | Publish fee in the mint's base units     |
| `authority` | `PublicKey`                        | Build only | Optional authority override              |

Derived by SDK:

- Registry config PDA
- Accepted payment token PDA

Example:

```ts
await registry.addPaymentToken({
  mint: paymentMint,
  feeAmount: 1_000_000n,
});
```

## updatePaymentToken

Updates whether a payment mint is active and changes its publish fee.

Execute signature:

```ts
updatePaymentToken(params: {
  mint: PublicKey;
  active: boolean;
  feeAmount: number | bigint | string | BN;
}): Promise<string>
```

Build signature:

```ts
updatePaymentTokenInstruction(params: {
  mint: PublicKey;
  active: boolean;
  feeAmount: number | bigint | string | BN;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type                               | Required   | Description                                  |
| ----------- | ---------------------------------- | ---------- | -------------------------------------------- |
| `mint`      | `PublicKey`                        | Yes        | SPL token mint                               |
| `active`    | `boolean`                          | Yes        | Whether this mint can be used for publishing |
| `feeAmount` | `number \| bigint \| string \| BN` | Yes        | Publish fee in base units                    |
| `authority` | `PublicKey`                        | Build only | Optional authority override                  |

Example:

```ts
await registry.updatePaymentToken({
  mint: paymentMint,
  active: true,
  feeAmount: 2_000_000n,
});
```

## removePaymentToken

Closes the accepted payment token config for a mint.

Execute signature:

```ts
removePaymentToken(params: {
  mint: PublicKey;
}): Promise<string>
```

Build signature:

```ts
removePaymentTokenInstruction(params: {
  mint: PublicKey;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type        | Required   | Description                 |
| ----------- | ----------- | ---------- | --------------------------- |
| `mint`      | `PublicKey` | Yes        | SPL token mint              |
| `authority` | `PublicKey` | Build only | Optional authority override |

Example:

```ts
await registry.removePaymentToken({
  mint: paymentMint,
});
```

## createPublishGrant

Creates a fee-exemption grant for a publisher.

Execute signature:

```ts
createPublishGrant(params: {
  publisher?: PublicKey;
  expiredAt?: number | bigint | string | BN | null;
  signers?: Signer[];
}): Promise<string>
```

Build signature:

```ts
createPublishGrantInstruction(params: {
  publisher?: PublicKey;
  expiredAt?: number | bigint | string | BN | null;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type                                       | Required     | Description                                                |
| ----------- | ------------------------------------------ | ------------ | ---------------------------------------------------------- |
| `publisher` | `PublicKey`                                | No           | Publisher receiving the grant. Defaults to provider wallet |
| `expiredAt` | `number \| bigint \| string \| BN \| null` | No           | Unix timestamp expiry. `null` means no expiry              |
| `signers`   | `Signer[]`                                 | Execute only | Extra local signers if publisher is not the wallet         |
| `authority` | `PublicKey`                                | Build only   | Optional authority override                                |

Signer notes:

- Authority must sign.
- Publisher must also sign.
- If authority and publisher are different browser wallets, use the build function and your own multi-signer flow.

Example:

```ts
await registry.createPublishGrant({
  publisher,
  expiredAt: null,
});
```

## updatePublishGrant

Updates an existing publish grant.

Execute signature:

```ts
updatePublishGrant(params: {
  publisher?: PublicKey;
  expiredAt?: number | bigint | string | BN | null;
  signers?: Signer[];
}): Promise<string>
```

Build signature:

```ts
updatePublishGrantInstruction(params: {
  publisher?: PublicKey;
  expiredAt?: number | bigint | string | BN | null;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name        | Type                                       | Required     | Description                                          |
| ----------- | ------------------------------------------ | ------------ | ---------------------------------------------------- |
| `publisher` | `PublicKey`                                | No           | Publisher grant account. Defaults to provider wallet |
| `expiredAt` | `number \| bigint \| string \| BN \| null` | No           | New Unix timestamp expiry. `null` means no expiry    |
| `signers`   | `Signer[]`                                 | Execute only | Extra local signers if publisher is not the wallet   |
| `authority` | `PublicKey`                                | Build only   | Optional authority override                          |

Example:

```ts
await registry.updatePublishGrant({
  publisher,
  expiredAt: BigInt(Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60),
});
```

## createGameAndRegister

Creates a PGL-1 game, creates the Registry game account, initializes Game Store config, and optionally creates a Game Store payment option.

Execute signature:

```ts
createGameAndRegister(params: {
  gameId: string;
  metadataUri: string;
  paymentMint: PublicKey;
  basePrice?: number | bigint | string | BN | null;
  mintToken?: PublicKey | null;
  includePublishGrant?: boolean;
  signers?: Signer[];
}): Promise<string>
```

Build signature:

```ts
createGameAndRegisterInstruction(params: {
  gameId: string;
  metadataUri: string;
  paymentMint: PublicKey;
  basePrice?: number | bigint | string | BN | null;
  mintToken?: PublicKey | null;
  includePublishGrant?: boolean;
  publisher?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name                  | Type                                       | Required     | Description                                             |
| --------------------- | ------------------------------------------ | ------------ | ------------------------------------------------------- |
| `gameId`              | `string`                                   | Yes          | Unique human-readable game id. Max 64 bytes             |
| `metadataUri`         | `string`                                   | Yes          | Metadata URI. Max 256 bytes                             |
| `paymentMint`         | `PublicKey`                                | Yes          | Mint used to pay the Registry publish fee               |
| `basePrice`           | `number \| bigint \| string \| BN \| null` | No           | Initial Game Store price in base units                  |
| `mintToken`           | `PublicKey \| null`                        | No           | Mint used for the Game Store payment option             |
| `includePublishGrant` | `boolean`                                  | No           | Include the publisher's grant PDA in remaining accounts |
| `signers`             | `Signer[]`                                 | Execute only | Extra local signers                                     |
| `publisher`           | `PublicKey`                                | Build only   | Optional publisher override                             |

Derived by SDK:

- Registry config PDA
- Registry accepted payment token PDA
- PGL-1 creator state PDA
- next PGL-1 game PDA
- Registry game PDA
- PGL-1 config PDA
- Game Store authorized source PDA
- Game Store authorized registry PDA
- Game Store game config PDA
- optional Game Store payment option PDA

App requirements:

- Publisher wallet signs.
- Publisher payment ATA must exist and hold enough publish-fee balance unless a valid grant is included.
- Treasury payment ATA must exist.
- If `basePrice` and `mintToken` are passed, the Store accepted payment token account must exist.

Free game example:

```ts
await registry.createGameAndRegister({
  gameId: "peridot-demo-game",
  metadataUri: "https://example.com/peridot-demo-game.json",
  paymentMint,
});
```

Paid game example:

```ts
await registry.createGameAndRegister({
  gameId: "peridot-demo-game",
  metadataUri: "https://example.com/peridot-demo-game.json",
  paymentMint,
  basePrice: 20_000_000n,
  mintToken: paymentMint,
});
```

Grant example:

```ts
await registry.createGameAndRegister({
  gameId: "peridot-granted-game",
  metadataUri: "https://example.com/peridot-granted-game.json",
  paymentMint,
  includePublishGrant: true,
});
```

## updateGameStatus

Updates the Registry status for a game.

Execute signature:

```ts
updateGameStatus(params: {
  registryGame: PublicKey;
  status: "active" | "suspended" | "banned" | number;
}): Promise<string>
```

Build signature:

```ts
updateGameStatusInstruction(params: {
  registryGame: PublicKey;
  status: "active" | "suspended" | "banned" | number;
  authority?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name           | Type                                            | Required   | Description                 |
| -------------- | ----------------------------------------------- | ---------- | --------------------------- |
| `registryGame` | `PublicKey`                                     | Yes        | Registry game PDA           |
| `status`       | `"active" \| "suspended" \| "banned" \| number` | Yes        | New status                  |
| `authority`    | `PublicKey`                                     | Build only | Optional authority override |

Status codes:

| Status      | Code |
| ----------- | ---- |
| `active`    | `0`  |
| `suspended` | `1`  |
| `banned`    | `2`  |

Allowed transitions:

- Active to Suspended
- Suspended to Active
- Active to Banned
- Suspended to Banned

Example:

```ts
await registry.updateGameStatus({
  registryGame,
  status: "suspended",
});
```

## closeRegistryGame

Closes a Registry game account. The game must be Suspended or Banned, and the connected wallet must be the PGL-1 game publisher.

Execute signature:

```ts
closeRegistryGame(params: {
  game: PublicKey;
  registryGame: PublicKey;
}): Promise<string>
```

Build signature:

```ts
closeRegistryGameInstruction(params: {
  game: PublicKey;
  registryGame: PublicKey;
  publisher?: PublicKey;
}): Promise<TransactionInstruction>
```

Parameters:

| Name           | Type        | Required   | Description                 |
| -------------- | ----------- | ---------- | --------------------------- |
| `game`         | `PublicKey` | Yes        | PGL-1 game account          |
| `registryGame` | `PublicKey` | Yes        | Registry game PDA           |
| `publisher`    | `PublicKey` | Build only | Optional publisher override |

Example:

```ts
await registry.closeRegistryGame({
  game,
  registryGame,
});
```

## Dashboard Pattern

A dashboard page should usually expose each instruction as:

- form inputs for function parameters
- a preview button that calls the `...Instruction` function
- a rendered account list from `ix.keys`
- an execute button that calls the execute function

Example:

```ts
const ix = await registry.addPaymentTokenInstruction({
  mint: paymentMint,
  feeAmount: 1_000_000n,
});

console.log(ix.programId.toBase58());
console.table(
  ix.keys.map((key) => ({
    pubkey: key.pubkey.toBase58(),
    signer: key.isSigner,
    writable: key.isWritable,
  })),
);
```

Then execute after user confirmation:

```ts
await registry.addPaymentToken({
  mint: paymentMint,
  feeAmount: 1_000_000n,
});
```

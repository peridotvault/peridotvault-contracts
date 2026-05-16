# Instruction Builders

The package now wraps every Registry instruction through `createRegistryClient`. See the [Registry docs](/solana/registry) for the primary frontend usage guide.

Lower-level Anchor examples are kept here as reference for teams that need custom transaction assembly.

For frontend execution today, use `@peridotvault/contracts` to get the correct IDL, program ID, and PDA helpers, then use Anchor to build or send the instruction.

## Frontend Setup

Install the app-side Solana dependencies:

```bash
pnpm add @peridotvault/contracts @coral-xyz/anchor @solana/web3.js @solana/spl-token
```

Create a Registry `Program` instance:

```ts
import { AnchorProvider, BN, Program, type Idl } from "@coral-xyz/anchor";
import {
  Connection,
  PublicKey,
  SystemProgram,
  Transaction
} from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
  REGISTRY_IDL,
  PGL1_IDL,
  findAuthorizedRegistryProgramPda,
  findAuthorizedSourceProgramPda,
  findCreatorStatePda,
  findGamePda,
  findGamePaymentOptionPda,
  findGameStoreConfigPda,
  findPglConfigPda,
  findPublishGrantPda,
  findRegistryConfigPda,
  findRegistryGamePda,
  findRegistryPaymentTokenPda,
  findStoreAcceptedPaymentTokenPda,
  getSolanaProgramId,
  type Pgl1,
  type Registry
} from "@peridotvault/contracts/solana";

const STATUS_ACTIVE = 0;
const STATUS_SUSPENDED = 1;
const STATUS_BANNED = 2;

export function createPeridotPrograms(wallet: AnchorProvider["wallet"]) {
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed"
  });

  const registryProgramId = getSolanaProgramId({
    environment: "testnet",
    program: "registry"
  });
  const pgl1ProgramId = getSolanaProgramId({
    environment: "testnet",
    program: "pgl1"
  });

  const registryProgram = new Program<Registry>(
    {
      ...REGISTRY_IDL,
      address: registryProgramId.toBase58()
    } as unknown as Idl,
    provider
  );

  const pgl1Program = new Program<Pgl1>(
    {
      ...PGL1_IDL,
      address: pgl1ProgramId.toBase58()
    } as unknown as Idl,
    provider
  );

  return { provider, registryProgram, pgl1Program };
}
```

Build an instruction without sending:

```ts
const ix = await registryProgram.methods
  .setTreasury(newTreasury)
  .accounts({ authority, config })
  .instruction();
```

Send from a frontend wallet:

```ts
const tx = new Transaction().add(ix);
const signature = await provider.sendAndConfirm(tx);
```

Or call Anchor directly:

```ts
const signature = await registryProgram.methods
  .setTreasury(newTreasury)
  .accounts({ authority, config })
  .rpc();
```

## Registry Instruction Index

| Instruction | Who signs | Purpose |
| --- | --- | --- |
| `initializeRegistry` | authority | Creates the singleton Registry config |
| `setTreasury` | authority | Updates the Registry treasury |
| `addPaymentToken` | authority | Adds a payment mint and publish fee |
| `updatePaymentToken` | authority | Enables/disables a payment mint and changes fee |
| `removePaymentToken` | authority | Closes a payment-token config |
| `createPublishGrant` | authority and publisher | Creates a fee-exemption grant |
| `updatePublishGrant` | authority and publisher | Updates a fee-exemption grant expiry |
| `createGameAndRegister` | publisher | Creates a PGL-1 game and Registry game |
| `updateGameStatus` | authority | Moves a Registry game between allowed statuses |
| `closeRegistryGame` | publisher | Closes a suspended or banned Registry game |

## initializeRegistry

Creates the singleton `registry_config` PDA.

Args:

- `treasury: PublicKey`

Accounts:

- `authority`: signer, payer
- `config`: `findRegistryConfigPda(registryProgram.programId)`
- `systemProgram`: `SystemProgram.programId`

```ts
export async function initializeRegistry(wallet: AnchorProvider["wallet"], treasury: PublicKey) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);

  return registryProgram.methods
    .initializeRegistry(treasury)
    .accounts({
      authority: provider.publicKey,
      config,
      systemProgram: SystemProgram.programId
    })
    .rpc();
}
```

## setTreasury

Updates the Registry treasury. Only the current Registry authority can call this.

Args:

- `treasury: PublicKey`

Accounts:

- `authority`: signer
- `config`: Registry config PDA

```ts
export async function setRegistryTreasury(wallet: AnchorProvider["wallet"], treasury: PublicKey) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);

  return registryProgram.methods
    .setTreasury(treasury)
    .accounts({
      authority: provider.publicKey,
      config
    })
    .rpc();
}
```

## addPaymentToken

Adds a supported payment mint and publish fee.

Args:

- `feeAmount: BN`

Accounts:

- `authority`: signer, payer
- `config`: Registry config PDA
- `mint`: SPL token mint
- `acceptedPaymentToken`: `findRegistryPaymentTokenPda(mint)`
- `systemProgram`: `SystemProgram.programId`

```ts
export async function addRegistryPaymentToken(
  wallet: AnchorProvider["wallet"],
  mint: PublicKey,
  feeAmount: BN
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
    mint,
    registryProgram.programId
  );

  return registryProgram.methods
    .addPaymentToken(feeAmount)
    .accounts({
      authority: provider.publicKey,
      config,
      mint,
      acceptedPaymentToken,
      systemProgram: SystemProgram.programId
    })
    .rpc();
}
```

## updatePaymentToken

Changes whether a payment mint is active and updates its publish fee.

Args:

- `active: boolean`
- `feeAmount: BN`

```ts
export async function updateRegistryPaymentToken(
  wallet: AnchorProvider["wallet"],
  mint: PublicKey,
  active: boolean,
  feeAmount: BN
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
    mint,
    registryProgram.programId
  );

  return registryProgram.methods
    .updatePaymentToken(active, feeAmount)
    .accounts({
      authority: provider.publicKey,
      config,
      mint,
      acceptedPaymentToken
    })
    .rpc();
}
```

## removePaymentToken

Closes a Registry payment-token config account.

Args: none

```ts
export async function removeRegistryPaymentToken(
  wallet: AnchorProvider["wallet"],
  mint: PublicKey
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
    mint,
    registryProgram.programId
  );

  return registryProgram.methods
    .removePaymentToken()
    .accounts({
      authority: provider.publicKey,
      config,
      mint,
      acceptedPaymentToken
    })
    .rpc();
}
```

## createPublishGrant

Creates a publish grant for a publisher. A valid grant lets `createGameAndRegister` skip the publish-fee transfer.

Args:

- `expiredAt: BN | null`

Accounts:

- `authority`: signer, payer
- `publisher`: signer
- `config`: Registry config PDA
- `publishGrant`: `findPublishGrantPda(publisher)`
- `systemProgram`: `SystemProgram.programId`

This instruction requires both authority and publisher signatures. In a normal frontend, that means the connected wallet must be the authority and publisher, or the app must coordinate multi-signer flow.

```ts
export async function createPublishGrant(
  wallet: AnchorProvider["wallet"],
  publisher: PublicKey,
  expiredAt: BN | null
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const [publishGrant] = findPublishGrantPda(publisher, registryProgram.programId);

  return registryProgram.methods
    .createPublishGrant(expiredAt)
    .accounts({
      authority: provider.publicKey,
      config,
      publisher,
      publishGrant,
      systemProgram: SystemProgram.programId
    })
    .rpc();
}
```

## updatePublishGrant

Updates a publisher grant expiry.

Args:

- `expiredAt: BN | null`

```ts
export async function updatePublishGrant(
  wallet: AnchorProvider["wallet"],
  publisher: PublicKey,
  expiredAt: BN | null
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const [publishGrant] = findPublishGrantPda(publisher, registryProgram.programId);

  return registryProgram.methods
    .updatePublishGrant(expiredAt)
    .accounts({
      authority: provider.publicKey,
      config,
      publisher,
      publishGrant
    })
    .rpc();
}
```

## createGameAndRegister

Creates a PGL-1 game, creates a Registry game, initializes Game Store config, and optionally sets a Game Store payment option.

Args:

- `gameId: string`
- `metadataUri: string`
- `basePrice: BN | null`
- `mintToken: PublicKey | null`

Important behavior:

- `gameId` must be non-empty and at most 64 bytes.
- `metadataUri` must be non-empty and at most 256 bytes.
- If no active publish grant is passed, the Registry charges the configured publish fee.
- If `basePrice` and `mintToken` are both provided, the instruction also configures the Game Store payment option.
- Transaction signing and sending are still controlled by your app or wallet.

```ts
export async function createGameAndRegister(wallet: AnchorProvider["wallet"], params: {
  gameId: string;
  metadataUri: string;
  paymentMint: PublicKey;
  basePrice?: BN;
  mintToken?: PublicKey;
  includePublishGrant?: boolean;
}) {
  const { registryProgram, pgl1Program, provider } = createPeridotPrograms(wallet);
  const publisher = provider.publicKey;

  const registryProgramId = registryProgram.programId;
  const pgl1ProgramId = getSolanaProgramId({
    environment: "testnet",
    program: "pgl1"
  });
  const storeProgramId = getSolanaProgramId({
    environment: "testnet",
    program: "store"
  });

  const [config] = findRegistryConfigPda(registryProgramId);
  const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
    params.paymentMint,
    registryProgramId
  );
  const [pglCreatorState] = findCreatorStatePda(publisher, pgl1ProgramId);

  let nextNonce = 0n;
  try {
    const creatorState = await pgl1Program.account.creatorState.fetch(pglCreatorState);
    nextNonce = BigInt(creatorState.nextNonce.toString());
  } catch {
    nextNonce = 0n;
  }

  const [game] = findGamePda(publisher, nextNonce, pgl1ProgramId);
  const [registryGame] = findRegistryGamePda(game, registryProgramId);
  const [pglConfig] = findPglConfigPda(pgl1ProgramId);
  const registryConfigAccount = await registryProgram.account.registryConfig.fetch(config);
  const pglConfigAccount = await pgl1Program.account.pglConfig.fetch(pglConfig);

  const [storeAuthorizedSourceProgram] = findAuthorizedSourceProgramPda(
    pgl1ProgramId,
    storeProgramId
  );
  const [storeAuthorizedRegistryProgram] = findAuthorizedRegistryProgramPda(
    registryProgramId,
    storeProgramId
  );
  const [storeGameStoreConfig] = findGameStoreConfigPda(game, storeProgramId);

  const publisherPaymentAccount = getAssociatedTokenAddressSync(
    params.paymentMint,
    publisher
  );
  const treasuryPaymentAccount = getAssociatedTokenAddressSync(
    params.paymentMint,
    registryConfigAccount.treasury
  );

  const remainingAccounts: {
    pubkey: PublicKey;
    isWritable: boolean;
    isSigner: boolean;
  }[] = [];

  if (params.includePublishGrant) {
    const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);
    remainingAccounts.push({
      pubkey: publishGrant,
      isWritable: false,
      isSigner: false
    });
  }

  if (params.basePrice && params.mintToken) {
    const [storeAcceptedPaymentToken] = findStoreAcceptedPaymentTokenPda(
      params.mintToken,
      storeProgramId
    );
    const [gamePaymentOption] = findGamePaymentOptionPda(
      game,
      params.mintToken,
      storeProgramId
    );

    remainingAccounts.push(
      { pubkey: storeAcceptedPaymentToken, isWritable: false, isSigner: false },
      { pubkey: gamePaymentOption, isWritable: true, isSigner: false }
    );
  }

  return registryProgram.methods
    .createGameAndRegister(
      params.gameId,
      params.metadataUri,
      params.basePrice ?? null,
      params.mintToken ?? null
    )
    .accounts({
      publisher,
      config,
      paymentMint: params.paymentMint,
      acceptedPaymentToken,
      publisherPaymentAccount,
      treasuryPaymentAccount,
      registryGame,
      game,
      pglCreatorState,
      pglConfig,
      pglTreasury: pglConfigAccount.treasury,
      pgl1Program: pgl1ProgramId,
      storeProgram: storeProgramId,
      storeAuthorizedSourceProgram,
      storeAuthorizedRegistryProgram,
      storeGameStoreConfig,
      registryProgram: registryProgramId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .remainingAccounts(remainingAccounts)
    .rpc();
}
```

## updateGameStatus

Updates a Registry game status.

Args:

- `status: number`

Allowed transitions:

- Active to Suspended
- Suspended to Active
- Active to Banned
- Suspended to Banned

Status constants:

```ts
const STATUS_ACTIVE = 0;
const STATUS_SUSPENDED = 1;
const STATUS_BANNED = 2;
```

```ts
export async function updateRegistryGameStatus(
  wallet: AnchorProvider["wallet"],
  registryGame: PublicKey,
  status: number
) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);

  return registryProgram.methods
    .updateGameStatus(status)
    .accounts({
      authority: provider.publicKey,
      config,
      registryGame
    })
    .rpc();
}
```

## closeRegistryGame

Closes a Registry game account. The game must be Suspended or Banned, and the signer must be the PGL-1 game publisher.

Args: none

Accounts:

- `publisher`: signer
- `config`: Registry config PDA
- `game`: PGL-1 game account
- `registryGame`: Registry game PDA
- `treasury`: Registry treasury

```ts
export async function closeRegistryGame(wallet: AnchorProvider["wallet"], params: {
  game: PublicKey;
  registryGame: PublicKey;
}) {
  const { registryProgram, provider } = createPeridotPrograms(wallet);
  const [config] = findRegistryConfigPda(registryProgram.programId);
  const configAccount = await registryProgram.account.registryConfig.fetch(config);

  return registryProgram.methods
    .closeRegistryGame()
    .accounts({
      publisher: provider.publicKey,
      config,
      game: params.game,
      registryGame: params.registryGame,
      treasury: configAccount.treasury
    })
    .rpc();
}
```

## Read: Fetch All Registry Games

Use this for a frontend list or dashboard table:

```ts
export async function fetchRegistryGames(wallet: AnchorProvider["wallet"]) {
  const { registryProgram } = createPeridotPrograms(wallet);
  const games = await registryProgram.account.registryGame.all();

  return games.map(({ publicKey, account }) => ({
    publicKey: publicKey.toBase58(),
    game: account.game.toBase58(),
    gameId: account.gameId,
    registeredAt: account.registeredAt.toNumber(),
    status: account.status
  }));
}
```

## Notes For A Dashboard UI

For an ICP Dashboard-style page, model each Registry instruction as a form:

- Resolve the environment and program IDs with `getSolanaProgramId`.
- Derive PDAs with the helpers from `@peridotvault/contracts/solana`.
- Show every resolved account before execution.
- Let the user choose between `Build instruction` and `Execute`.
- Use `.instruction()` for preview/build mode.
- Use `.rpc()` or `provider.sendAndConfirm(new Transaction().add(ix))` for execution mode.
- Never hide required signers. `createPublishGrant` and `updatePublishGrant` require both `authority` and `publisher`.

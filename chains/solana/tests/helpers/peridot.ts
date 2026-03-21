import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { createHash } from "crypto";

import { Factory } from "../../target/types/factory";
import { GameStore } from "../../target/types/game_store";
import { Pgc1 } from "../../target/types/pgc1";
import { Registry } from "../../target/types/registry";

export const REGISTRY_STATE_SEED = Buffer.from("registry_state");
export const STORE_STATE_SEED = Buffer.from("game_store_state");
export const FACTORY_STATE_SEED = Buffer.from("factory_state");
export const FACTORY_MINT_SEED = Buffer.from("factory_mint");
export const GAME_STATE_SEED = Buffer.from("game_state");
export const GAME_AUTHORITY_SEED = Buffer.from("game_authority");
export const MINTER_AUTH_SEED = Buffer.from("minter_auth");
export const LICENSE_SEED = Buffer.from("license");

export const STATUS_PENDING = 0;
export const STATUS_APPROVED = 1;

export const DEFAULT_PLATFORM_FEE_BPS = 1000;
export const UPDATED_PLATFORM_FEE_BPS = 750;
export const DEFAULT_REGISTRATION_FEE = 5_000_000;
export const DEFAULT_GAME_PRICE = 20_000_000;
export const DEFAULT_GAME_DISCOUNT_BPS = 1_500;
export const PAYMENT_DECIMALS = 6;

export const TEST_GAME_ID = "peridot-localnet-alpha";
export const TEST_METADATA_URI = "https://peridot.local/metadata/peridot-localnet-alpha.json";

type NodeWallet = anchor.Wallet & { payer: Keypair };

export type WorkspacePrograms = {
  factoryProgram: Program<Factory>;
  pgcProgram: Program<Pgc1>;
  registryProgram: Program<Registry>;
  storeProgram: Program<GameStore>;
};

export type BaseFixture = WorkspacePrograms & {
  provider: anchor.AnchorProvider;
  payer: Keypair;
  governance: Keypair;
  nextGovernance: Keypair;
  treasury: Keypair;
  nextTreasury: Keypair;
  publisher: Keypair;
  gamer: Keypair;
  paymentMint: PublicKey;
  publisherPaymentTokenAccount: PublicKey;
  gamerPaymentTokenAccount: PublicKey;
  treasuryPaymentTokenAccount: PublicKey;
  registryStatePda: PublicKey;
  storeStatePda: PublicKey;
  factoryStatePda: PublicKey;
};

export type GameFixture = {
  gameId: string;
  metadataUri: string;
  mintPda: PublicKey;
  gameStatePda: PublicKey;
  gameAuthorityPda: PublicKey;
  publisherMinterAuthPda: PublicKey;
  storeMinterAuthPda: PublicKey;
};

let baseFixturePromise: Promise<BaseFixture> | null = null;

function providerWallet(provider: anchor.AnchorProvider): NodeWallet {
  return provider.wallet as NodeWallet;
}

function workspaceProgram<T extends anchor.Idl>(name: string): Program<T> {
  const workspace = anchor.workspace as Record<string, Program<T>>;
  return (workspace[name] ?? workspace[name.toLowerCase()]) as Program<T>;
}

function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

function sha256Seed(value: string): Buffer {
  return createHash("sha256").update(value).digest();
}

async function accountExists(
  connection: anchor.web3.Connection,
  address: PublicKey,
): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

async function fundSigner(
  provider: anchor.AnchorProvider,
  recipient: PublicKey,
  lamports = 2 * anchor.web3.LAMPORTS_PER_SOL,
): Promise<void> {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: provider.publicKey,
      toPubkey: recipient,
      lamports,
    }),
  );
  await provider.sendAndConfirm(tx);
}

async function maybeFundSigner(
  provider: anchor.AnchorProvider,
  recipient: Keypair,
): Promise<void> {
  const balance = await provider.connection.getBalance(recipient.publicKey);
  if (balance === 0) {
    await fundSigner(provider, recipient.publicKey);
  }
}

async function createPaymentMintAndAccounts(
  provider: anchor.AnchorProvider,
  treasury: Keypair,
  publisher: Keypair,
  gamer: Keypair,
): Promise<{
  paymentMint: PublicKey;
  treasuryPaymentTokenAccount: PublicKey;
  publisherPaymentTokenAccount: PublicKey;
  gamerPaymentTokenAccount: PublicKey;
}> {
  const payer = providerWallet(provider).payer;
  const paymentMint = await createMint(
    provider.connection,
    payer,
    provider.publicKey,
    null,
    PAYMENT_DECIMALS,
  );

  const treasuryPaymentTokenAccount = (
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      paymentMint,
      treasury.publicKey,
    )
  ).address;
  const publisherPaymentTokenAccount = (
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      paymentMint,
      publisher.publicKey,
    )
  ).address;
  const gamerPaymentTokenAccount = (
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      paymentMint,
      gamer.publicKey,
    )
  ).address;

  await mintTo(
    provider.connection,
    payer,
    paymentMint,
    publisherPaymentTokenAccount,
    payer,
    1_000_000_000,
  );
  await mintTo(
    provider.connection,
    payer,
    paymentMint,
    gamerPaymentTokenAccount,
    payer,
    1_000_000_000,
  );

  return {
    paymentMint,
    treasuryPaymentTokenAccount,
    publisherPaymentTokenAccount,
    gamerPaymentTokenAccount,
  };
}

async function initializeBaseFixture(): Promise<BaseFixture> {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const registryProgram = workspaceProgram<Registry>("Registry");
  const storeProgram = workspaceProgram<GameStore>("GameStore");
  const factoryProgram = workspaceProgram<Factory>("Factory");
  const pgcProgram = workspaceProgram<Pgc1>("Pgc1");

  const payer = providerWallet(provider).payer;
  const governance = payer;
  const nextGovernance = Keypair.generate();
  const treasury = Keypair.generate();
  const nextTreasury = Keypair.generate();
  const publisher = Keypair.generate();
  const gamer = Keypair.generate();

  await maybeFundSigner(provider, nextGovernance);
  await maybeFundSigner(provider, publisher);
  await maybeFundSigner(provider, gamer);

  const registryStatePda = derivePda([REGISTRY_STATE_SEED], registryProgram.programId);
  const storeStatePda = derivePda([STORE_STATE_SEED], storeProgram.programId);
  const factoryStatePda = derivePda([FACTORY_STATE_SEED], factoryProgram.programId);

  const {
    paymentMint,
    treasuryPaymentTokenAccount,
    publisherPaymentTokenAccount,
    gamerPaymentTokenAccount,
  } = await createPaymentMintAndAccounts(provider, treasury, publisher, gamer);

  if (!(await accountExists(provider.connection, registryStatePda))) {
    await registryProgram.methods
      .initialize(
        governance.publicKey,
        treasury.publicKey,
        factoryStatePda,
        new anchor.BN(DEFAULT_REGISTRATION_FEE),
        paymentMint,
      )
      .accounts({
        payer: provider.publicKey,
        registryState: registryStatePda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();
  }

  if (!(await accountExists(provider.connection, storeStatePda))) {
    await storeProgram.methods
      .initialize(
        governance.publicKey,
        treasury.publicKey,
        registryStatePda,
        DEFAULT_PLATFORM_FEE_BPS,
      )
      .accounts({
        payer: provider.publicKey,
        storeState: storeStatePda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();
  }

  if (!(await accountExists(provider.connection, factoryStatePda))) {
    await factoryProgram.methods
      .initialize(governance.publicKey, registryStatePda, storeStatePda)
      .accounts({
        payer: provider.publicKey,
        factoryState: factoryStatePda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();
  }

  return {
    provider,
    payer,
    governance,
    nextGovernance,
    treasury,
    nextTreasury,
    publisher,
    gamer,
    paymentMint,
    publisherPaymentTokenAccount,
    gamerPaymentTokenAccount,
    treasuryPaymentTokenAccount,
    registryStatePda,
    storeStatePda,
    factoryStatePda,
    registryProgram,
    storeProgram,
    factoryProgram,
    pgcProgram,
  };
}

export async function setupPeridotFixture(): Promise<BaseFixture> {
  if (!baseFixturePromise) {
    baseFixturePromise = initializeBaseFixture();
  }
  return baseFixturePromise;
}

export function deriveGameFixture(base: BaseFixture, gameId = TEST_GAME_ID): GameFixture {
  const mintPda = derivePda(
    [FACTORY_MINT_SEED, sha256Seed(gameId)],
    base.factoryProgram.programId,
  );
  const gameStatePda = derivePda(
    [GAME_STATE_SEED, Buffer.from(gameId)],
    base.pgcProgram.programId,
  );
  const gameAuthorityPda = derivePda(
    [GAME_AUTHORITY_SEED, gameStatePda.toBuffer()],
    base.pgcProgram.programId,
  );
  const publisherMinterAuthPda = derivePda(
    [MINTER_AUTH_SEED, gameStatePda.toBuffer(), base.publisher.publicKey.toBuffer()],
    base.pgcProgram.programId,
  );
  const storeMinterAuthPda = derivePda(
    [MINTER_AUTH_SEED, gameStatePda.toBuffer(), base.storeStatePda.toBuffer()],
    base.pgcProgram.programId,
  );

  return {
    gameId,
    metadataUri: TEST_METADATA_URI,
    mintPda,
    gameStatePda,
    gameAuthorityPda,
    publisherMinterAuthPda,
    storeMinterAuthPda,
  };
}

export async function ensureGameCreated(base: BaseFixture): Promise<GameFixture> {
  const game = deriveGameFixture(base);

  if (!(await accountExists(base.provider.connection, game.gameStatePda))) {
    await base.factoryProgram.methods
      .createGame(game.gameId, game.metadataUri, base.paymentMint)
      .accounts({
        publisher: base.publisher.publicKey,
        factoryState: base.factoryStatePda,
        mint: game.mintPda,
        pgcProgram: base.pgcProgram.programId,
        pgcGameState: game.gameStatePda,
        pgcGameAuthority: game.gameAuthorityPda,
        publisherMinterAuth: game.publisherMinterAuthPda,
        gameStoreMinterAuth: game.storeMinterAuthPda,
        registryProgram: base.registryProgram.programId,
        registryState: base.registryStatePda,
        treasury: base.treasury.publicKey,
        gameStore: base.storeStatePda,
        publisherFeeTokenAccount: base.publisherPaymentTokenAccount,
        treasuryFeeTokenAccount: base.treasuryPaymentTokenAccount,
        feePaymentMint: base.paymentMint,
        paymentTokenProgram: TOKEN_PROGRAM_ID,
        licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([base.publisher])
      .rpc();
  }

  return game;
}

export async function approveGame(base: BaseFixture, gameId = TEST_GAME_ID): Promise<void> {
  const registryState = (await base.registryProgram.account.registryState.fetch(
    base.registryStatePda,
  )) as any;
  const game = registryState.games.find((entry: any) => entry.gameId === gameId);
  if (!game || game.status === STATUS_APPROVED) {
    return;
  }

  await base.registryProgram.methods
    .setStatus(gameId, STATUS_APPROVED)
    .accounts({
      admin: base.governance.publicKey,
      registryState: base.registryStatePda,
    } as any)
    .rpc();
}

export async function ensurePriceConfigured(base: BaseFixture): Promise<void> {
  const game = await ensureGameCreated(base);
  await approveGame(base, game.gameId);

  const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;
  const existingPrice = storeState.prices.find((entry: any) => entry.gameId === game.gameId);

  if (!existingPrice) {
    await base.storeProgram.methods
      .setPrice(
        game.gameId,
        new anchor.BN(DEFAULT_GAME_PRICE),
        base.paymentMint,
      )
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcGameState: game.gameStatePda,
        currencyMint: base.paymentMint,
      } as any)
      .signers([base.publisher])
      .rpc();
  }

  const refreshedStoreState = (await base.storeProgram.account.storeState.fetch(
    base.storeStatePda,
  )) as any;
  const currentPrice = refreshedStoreState.prices.find(
    (entry: any) => entry.gameId === game.gameId,
  );

  if (!currentPrice || currentPrice.discountBps !== DEFAULT_GAME_DISCOUNT_BPS) {
    await base.storeProgram.methods
      .setDiscount(game.gameId, DEFAULT_GAME_DISCOUNT_BPS)
      .accounts({
        publisher: base.publisher.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcGameState: game.gameStatePda,
      } as any)
      .signers([base.publisher])
      .rpc();
  }
}

export async function buyGameForGamer(base: BaseFixture): Promise<{
  game: GameFixture;
  licensePda: PublicKey;
  userGameTokenAccount: PublicKey;
  storeVaultTokenAccount: PublicKey;
}> {
  await ensurePriceConfigured(base);

  const game = deriveGameFixture(base);
  const licensePda = derivePda(
    [LICENSE_SEED, game.gameStatePda.toBuffer(), base.gamer.publicKey.toBuffer()],
    base.pgcProgram.programId,
  );
  const userGameTokenAccount = getAssociatedTokenAddressSync(
    game.mintPda,
    base.gamer.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
  );
  const storeVaultTokenAccount = getAssociatedTokenAddressSync(
    base.paymentMint,
    base.storeStatePda,
    true,
    TOKEN_PROGRAM_ID,
  );

  if (!(await accountExists(base.provider.connection, licensePda))) {
    await base.storeProgram.methods
      .buyGame(game.gameId)
      .accounts({
        buyer: base.gamer.publicKey,
        storeState: base.storeStatePda,
        registryState: base.registryStatePda,
        pgcProgram: base.pgcProgram.programId,
        pgcGameState: game.gameStatePda,
        gameAuthority: game.gameAuthorityPda,
        storeMinterAuth: game.storeMinterAuthPda,
        licenseAccount: licensePda,
        userGameTokenAccount,
        gameMint: game.mintPda,
        paymentMint: base.paymentMint,
        buyerPaymentTokenAccount: base.gamerPaymentTokenAccount,
        treasuryTokenAccount: base.treasuryPaymentTokenAccount,
        storeVaultTokenAccount,
        paymentTokenProgram: TOKEN_PROGRAM_ID,
        licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([base.gamer])
      .rpc();
  }

  return {
    game,
    licensePda,
    userGameTokenAccount,
    storeVaultTokenAccount,
  };
}

export async function getCatalogWithPrices(base: BaseFixture): Promise<
  Array<{
    gameId: string;
    contractAddress: PublicKey;
    status: number;
    price: number | null;
    discountBps: number | null;
    finalPrice: number | null;
  }>
> {
  const registryState = (await base.registryProgram.account.registryState.fetch(
    base.registryStatePda,
  )) as any;
  const storeState = (await base.storeProgram.account.storeState.fetch(base.storeStatePda)) as any;

  return registryState.games.map((game: any) => {
    const priceConfig = storeState.prices.find(
      (entry: any) => entry.gameId === game.gameId,
    );
    const price = priceConfig ? Number(priceConfig.price.toString()) : null;
    const discountBps = priceConfig ? priceConfig.discountBps : null;
    const finalPrice =
      priceConfig === undefined || price === null
        ? null
        : price - Math.floor((price * discountBps!) / 10_000);

    return {
      gameId: game.gameId,
      contractAddress: game.contractAddress as PublicKey,
      status: game.status,
      price,
      discountBps,
      finalPrice,
    };
  });
}

export async function listOwnedGames(base: BaseFixture, owner: PublicKey): Promise<
  Array<{
    gameId: string;
    contractAddress: PublicKey;
    status: number;
    finalPrice: number | null;
    licenseAddress: PublicKey;
  }>
> {
  const catalog = await getCatalogWithPrices(base);
  const now = Math.floor(Date.now() / 1000);
  const ownedGames: Array<{
    gameId: string;
    contractAddress: PublicKey;
    status: number;
    finalPrice: number | null;
    licenseAddress: PublicKey;
  }> = [];

  for (const game of catalog) {
    const licenseAddress = derivePda(
      [LICENSE_SEED, game.contractAddress.toBuffer(), owner.toBuffer()],
      base.pgcProgram.programId,
    );
    if (!(await accountExists(base.provider.connection, licenseAddress))) {
      continue;
    }

    const license = (await base.pgcProgram.account.licenseAccount.fetch(
      licenseAddress,
    )) as any;
    const expiresAt = Number(license.expiresAt.toString());
    if (expiresAt === 0 || expiresAt > now) {
      ownedGames.push({
        gameId: game.gameId,
        contractAddress: game.contractAddress,
        status: game.status,
        finalPrice: game.finalPrice,
        licenseAddress,
      });
    }
  }

  return ownedGames;
}

export async function paymentTokenBalance(
  base: BaseFixture,
  address: PublicKey,
): Promise<number> {
  const account = await getAccount(base.provider.connection, address);
  return Number(account.amount);
}

export async function licenseTokenBalance(
  base: BaseFixture,
  address: PublicKey,
): Promise<number> {
  const account = await getAccount(
    base.provider.connection,
    address,
    undefined,
    TOKEN_2022_PROGRAM_ID,
  );
  return Number(account.amount);
}

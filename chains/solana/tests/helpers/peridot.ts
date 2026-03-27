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

import { GameStore } from "../../target/types/game_store";
import { Pgc1 } from "../../target/types/pgc1";
import { Registry } from "../../target/types/registry";

export const GLOBAL_STATE_SEED = Buffer.from("global_program_state");
export const REGISTRY_STATE_SEED = Buffer.from("registry_state");
export const STORE_STATE_SEED = Buffer.from("game_store_state");
export const GAME_STATE_SEED = Buffer.from("game_state");
export const GAME_AUTHORITY_SEED = Buffer.from("game_authority");
export const MINTER_AUTH_SEED = Buffer.from("minter_auth");
export const GAME_SEED = Buffer.from("game");
export const LICENSE_SEED = Buffer.from("license");
export const PRICE_SEED = Buffer.from("price");
export const BALANCE_SEED = Buffer.from("balance");

export const STATUS_PENDING = 0;
export const STATUS_APPROVED = 1;

export const DEFAULT_PLATFORM_FEE_BPS = 1000;
export const UPDATED_PLATFORM_FEE_BPS = 750;
export const DEFAULT_REGISTRATION_FEE = 5_000_000;
export const DEFAULT_GAME_PRICE = 20_000_000;
export const DEFAULT_GAME_DISCOUNT_BPS = 1500;
export const PAYMENT_DECIMALS = 6;

export const TEST_GAME_ID = "peridot-localnet-alpha";
export const TEST_METADATA_URI = "https://peridot.local/metadata/peridot-localnet-alpha.json";

type NodeWallet = anchor.Wallet & { payer: Keypair };

export type WorkspacePrograms = {
  pgc1Program: Program<Pgc1>;
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
  registryStatePda: PublicKey;
  storeStatePda: PublicKey;
  pgcGlobalStatePda: PublicKey;
};

export type GameFixture = {
  gameId: string;
  metadataUri: string;
  gameStatePda: PublicKey;
  gameAuthorityPda: PublicKey;
  publisherMinterAuthPda: PublicKey;
  gameRegistrationPda: PublicKey;
  pricePda: PublicKey;
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

async function createPaymentMint(
  provider: anchor.AnchorProvider,
): Promise<PublicKey> {
  const payer = providerWallet(provider).payer;
  return await createMint(
    provider.connection,
    payer,
    provider.publicKey,
    null,
    PAYMENT_DECIMALS,
  );
}

async function initializeBaseFixture(): Promise<BaseFixture> {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const registryProgram = workspaceProgram<Registry>("Registry");
  const storeProgram = workspaceProgram<GameStore>("GameStore");
  const pgc1Program = workspaceProgram<Pgc1>("Pgc1");

  const payer = providerWallet(provider).payer;
  const governance = payer;
  const nextGovernance = Keypair.generate();
  const treasury = Keypair.generate();
  const nextTreasury = Keypair.generate();
  const publisher = Keypair.generate();
  const gamer = Keypair.generate();

  await maybeFundSigner(provider, nextGovernance);
  await maybeFundSigner(provider, nextTreasury);
  await maybeFundSigner(provider, publisher);
  await maybeFundSigner(provider, gamer);

  const registryStatePda = derivePda([REGISTRY_STATE_SEED], registryProgram.programId);
  const storeStatePda = derivePda([STORE_STATE_SEED], storeProgram.programId);
  const pgcGlobalStatePda = derivePda([GLOBAL_STATE_SEED], pgc1Program.programId);

  const paymentMint = await createPaymentMint(provider);

  // Initialize programs if needed
  if (!(await accountExists(provider.connection, registryStatePda))) {
    await registryProgram.methods
      .initialize(governance.publicKey, treasury.publicKey)
      .accounts({
        // payer: provider.publicKey, (implicitly handles via wallet)
        registryState: registryStatePda,
        sysProg: anchor.web3.SystemProgram.programId,
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
        storeState: storeStatePda,
        sysProg: anchor.web3.SystemProgram.programId,
      } as any)
      .rpc();
  }

  if (!(await accountExists(provider.connection, pgcGlobalStatePda))) {
    await pgc1Program.methods
      .initializeProgram(
        governance.publicKey,
        registryProgram.programId,
        storeProgram.programId,
      )
      .accounts({
        globalState: pgcGlobalStatePda,
        sysProg: anchor.web3.SystemProgram.programId,
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
    registryStatePda,
    storeStatePda,
    pgcGlobalStatePda,
    registryProgram,
    storeProgram,
    pgc1Program,
  };
}

export async function setupPeridotFixture(): Promise<BaseFixture> {
  if (!baseFixturePromise) {
    baseFixturePromise = initializeBaseFixture();
  }
  return baseFixturePromise;
}

export function deriveGameFixture(base: BaseFixture, gameId = TEST_GAME_ID): GameFixture {
  const gameStatePda = derivePda(
    [GAME_STATE_SEED, Buffer.from(gameId)],
    base.pgc1Program.programId,
  );
  const gameAuthorityPda = derivePda(
    [GAME_AUTHORITY_SEED, gameStatePda.toBuffer()],
    base.pgc1Program.programId,
  );
  const publisherMinterAuthPda = derivePda(
    [MINTER_AUTH_SEED, gameStatePda.toBuffer(), base.publisher.publicKey.toBuffer()],
    base.pgc1Program.programId,
  );
  const gameRegistrationPda = derivePda(
    [GAME_SEED, Buffer.from(gameId)],
    base.registryProgram.programId,
  );
  const pricePda = derivePda(
    [PRICE_SEED, gameStatePda.toBuffer()],
    base.storeProgram.programId,
  );

  return {
    gameId,
    metadataUri: TEST_METADATA_URI,
    gameStatePda,
    gameAuthorityPda,
    publisherMinterAuthPda,
    gameRegistrationPda,
    pricePda,
  };
}

export async function ensureGameCreated(base: BaseFixture, gameId = TEST_GAME_ID): Promise<GameFixture> {
  const game = deriveGameFixture(base, gameId);
  const mintKp = Keypair.generate();

    if (!(await accountExists(base.provider.connection, game.gameStatePda))) {
      await base.pgc1Program.methods
        .createGame(
          game.gameId,
          base.publisher.publicKey,
          game.metadataUri,
          new anchor.BN(DEFAULT_GAME_PRICE),
          SystemProgram.programId,
        )
        .accounts({
          payer: base.publisher.publicKey,
          mint: mintKp.publicKey,
          gameState: game.gameStatePda,
          gameAuthority: game.gameAuthorityPda,
          publisherAccount: base.publisher.publicKey,
          publisherMinterAuth: game.publisherMinterAuthPda,
          globalState: base.pgcGlobalStatePda,
          registryProgram: base.registryProgram.programId,
          registryState: base.registryStatePda,
          gameRegistration: game.gameRegistrationPda,
          gameStoreProgram: base.storeProgram.programId,
          storeState: base.storeStatePda,
          priceAccount: game.pricePda,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          sysProg: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([base.publisher, mintKp])
        .rpc();
    }

  return game;
}

export async function approveGame(base: BaseFixture, gameId = TEST_GAME_ID): Promise<void> {
  const game = deriveGameFixture(base, gameId);
  const reg = await base.registryProgram.account.gameRegistration.fetch(game.gameRegistrationPda);
  if (reg.status === STATUS_APPROVED) return;

  await base.registryProgram.methods
    .setStatus(gameId, STATUS_APPROVED)
    .accounts({
      admin: base.governance.publicKey,
      registryState: base.registryStatePda,
      gameRegistration: game.gameRegistrationPda,
    } as any)
    .signers([base.governance])
    .rpc();
}

export async function ensurePriceConfigured(base: BaseFixture): Promise<void> {
  const game = await ensureGameCreated(base);
  await approveGame(base, game.gameId);
}

export async function buyGameForGamer(base: BaseFixture, gameId = TEST_GAME_ID): Promise<{
  game: GameFixture;
  licensePda: PublicKey;
  userGameTokenAccount: PublicKey;
}> {
  await ensurePriceConfigured(base);

  const game = deriveGameFixture(base, gameId);
  const licensePda = derivePda(
    [LICENSE_SEED, game.gameStatePda.toBuffer(), base.gamer.publicKey.toBuffer()],
    base.pgc1Program.programId,
  );
  
  const userGameTokenAccount = getAssociatedTokenAddressSync(
    (await base.pgc1Program.account.gameState.fetch(game.gameStatePda)).mint,
    base.gamer.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
  );

  const priceAccount = await base.storeProgram.account.priceAccount.fetch(game.pricePda);
  const balancePda = derivePda(
    [BALANCE_SEED, base.publisher.publicKey.toBuffer(), priceAccount.currency.toBuffer()],
    base.storeProgram.programId
  );

    if (!(await accountExists(base.provider.connection, licensePda))) {
      await base.storeProgram.methods
        .buyGame()
        .accounts({
          buyer: base.gamer.publicKey,
          storeState: base.storeStatePda,
          treasury: base.treasury.publicKey,
          pgcGameState: game.gameStatePda,
          priceAccount: game.pricePda,
          publisherBalanceAccount: balancePda,
          sysProg: anchor.web3.SystemProgram.programId,
        } as any)
        .signers([base.gamer])
        .rpc();
    }

  return {
    game,
    licensePda,
    userGameTokenAccount,
  };
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

export async function getCatalogWithPrices(base: BaseFixture): Promise<any[]> {
  const registrations = await base.registryProgram.account.gameRegistration.all();
  const results = [];
  for (const reg of registrations) {
    const game = reg.account;
    const gameId = game.gameId;
    const fixture = deriveGameFixture(base, gameId);
    let price = null;
    let discountBps = null;
    try {
      const priceAccount = await base.storeProgram.account.priceAccount.fetch(fixture.pricePda);
      price = Number(priceAccount.price.toString());
      discountBps = priceAccount.discountBps;
    } catch (e) {}

    results.push({
      gameId,
      status: game.status,
      price,
      discountBps,
      finalPrice: price === null ? null : price - Math.floor((price * (discountBps ?? 0)) / 10000),
    });
  }
  return results;
}

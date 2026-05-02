import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

export const PGL1_PROGRAM_ID = new PublicKey("AHpAEMxUEk4Um3E6PgXxFQiiTBhSQP9Ej2Sy77Y7WU6H");
export const REGISTRY_PROGRAM_ID = new PublicKey("2H2RfFxMYxh6njAJNekPacK671DL9q2W89YjiQhAM4ut");
export const STORE_PROGRAM_ID = new PublicKey("FHxSLLvsy8z7rWmP3451EWKQd5QMxri9R8ug73wcWEJC");

export const DEFAULT_PLATFORM_FEE_BPS = 1_000;
export const DEFAULT_REFERRAL_BPS = 200;
export const DEFAULT_MAX_REFERRAL_BPS = 5_000;
export const DEFAULT_REGISTRY_FEE = 1_000;
export const DEFAULT_GAME_PRICE = 20_000_000;
export const PAYMENT_DECIMALS = 6;
export const PUBLISHER_MINT_AMOUNT = 1_000_000_000;

export const ROLE_SOURCE = 0;
export const ROLE_REGISTRY = 1;

export const STATUS_ACTIVE = 0;
export const STATUS_SUSPENDED = 1;
export const STATUS_BANNED = 2;

declare const require: any;

export type GameFixture = {
  gameId: string;
  metadataUri: string;
  gamePda: PublicKey;
  creatorStatePda: PublicKey;
  registryGamePda: PublicKey;
  publisher: Keypair;
};

export type StoreGameFixture = {
  gameStoreConfigPda: PublicKey;
  gamePaymentOptionPda: PublicKey;
  payment_mint: PublicKey;
  basePrice: number;
};

type NodeWallet = anchor.Wallet & { payer: Keypair };

export type BaseFixture = {
  provider: anchor.AnchorProvider;
  authority: Keypair;
  publisher: Keypair;
  gamer: Keypair;
  pglProgram: any;
  registry_program: any;
  storeProgram: any;
  payment_mint: PublicKey;
  pglConfigPda: PublicKey;
  registryConfigPda: PublicKey;
  storeConfigPda: PublicKey;
  authorizedSourceProgramPda: PublicKey;
  authorizedRegistryProgramPda: PublicKey;
  storeActorAuthorizedPda: PublicKey;
  registryAcceptedPaymentTokenPda: PublicKey;
  storeAcceptedPaymentTokenPda: PublicKey;
};

let baseFixturePromise: Promise<BaseFixture> | null = null;

function providerWallet(provider: anchor.AnchorProvider): NodeWallet {
  return provider.wallet as NodeWallet;
}

export function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

function u64LeBuffer(value: bigint): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
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

async function initializeBaseFixture(): Promise<BaseFixture> {
  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const pglIdl = require("../../target/idl/pgl1.json");
  const registryIdl = require("../../target/idl/registry.json");
  const storeIdl = require("../../target/idl/game_store.json");

  const pglProgram = new Program(pglIdl, provider);
  const registryProgram = new Program(registryIdl, provider);
  const storeProgram = new Program(storeIdl, provider);

  const authority = providerWallet(provider).payer;
  const publisher = Keypair.generate();
  const gamer = Keypair.generate();

  await maybeFundSigner(provider, publisher);
  await maybeFundSigner(provider, gamer);

  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  if (!(await accountExists(provider.connection, pglConfigPda))) {
    await pglProgram.methods
      .initializePgl(authority.publicKey, new anchor.BN(0))
      .accounts({
        authority: authority.publicKey,
        pglConfig: pglConfigPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, registryConfigPda))) {
    await registryProgram.methods
      .initializeRegistry(authority.publicKey)
      .accounts({
        authority: authority.publicKey,
        config: registryConfigPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, storeConfigPda))) {
    await storeProgram.methods
      .initializeStore(
        authority.publicKey,
        DEFAULT_PLATFORM_FEE_BPS,
        DEFAULT_REFERRAL_BPS,
        DEFAULT_MAX_REFERRAL_BPS,
        authority.publicKey,
      )
      .accounts({
        authority: authority.publicKey,
        store_config: storeConfigPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  const paymentMint = await createMint(
    provider.connection,
    authority,
    authority.publicKey,
    null,
    PAYMENT_DECIMALS,
  );

  const registryAcceptedPaymentTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
    registryProgram.programId,
  );
  const storeAcceptedPaymentTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
    storeProgram.programId,
  );

  if (!(await accountExists(provider.connection, registryAcceptedPaymentTokenPda))) {
    await registryProgram.methods
      .addPaymentToken(new anchor.BN(DEFAULT_REGISTRY_FEE))
      .accounts({
        authority: authority.publicKey,
        config: registryConfigPda,
        mint: paymentMint,
        accepted_payment_token: registryAcceptedPaymentTokenPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, storeAcceptedPaymentTokenPda))) {
    await storeProgram.methods
      .addPaymentToken()
      .accounts({
        authority: authority.publicKey,
        store_config: storeConfigPda,
        mint: paymentMint,
        accepted_payment_token: storeAcceptedPaymentTokenPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  const authorizedSourceProgramPda = derivePda(
    [Buffer.from("authorized_program"), pglProgram.programId.toBuffer()],
    storeProgram.programId,
  );
  const authorizedRegistryProgramPda = derivePda(
    [Buffer.from("authorized_program"), registryProgram.programId.toBuffer()],
    storeProgram.programId,
  );

  if (!(await accountExists(provider.connection, authorizedSourceProgramPda))) {
    await storeProgram.methods
      .addAuthorizedProgram(ROLE_SOURCE)
      .accounts({
        authority: authority.publicKey,
        store_config: storeConfigPda,
        programId: pglProgram.programId,
        authorizedProgram: authorizedSourceProgramPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, authorizedRegistryProgramPda))) {
    await storeProgram.methods
      .addAuthorizedProgram(ROLE_REGISTRY)
      .accounts({
        authority: authority.publicKey,
        store_config: storeConfigPda,
        programId: registryProgram.programId,
        authorizedProgram: authorizedRegistryProgramPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  const storeActorAuthorizedPda = derivePda(
    [Buffer.from("authorized_actor"), authority.publicKey.toBuffer()],
    pglProgram.programId,
  );
  if (!(await accountExists(provider.connection, storeActorAuthorizedPda))) {
    await pglProgram.methods
      .addAuthorizedActor()
      .accounts({
        authority: authority.publicKey,
        actor: authority.publicKey,
        pglConfig: pglConfigPda,
        authorized_actor: storeActorAuthorizedPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  await getOrCreateAssociatedTokenAccount(
    provider.connection,
    authority,
    paymentMint,
    authority.publicKey,
  );

  const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    authority,
    paymentMint,
    publisher.publicKey,
  );

  await mintTo(
    provider.connection,
    authority,
    paymentMint,
    publisherPaymentAta.address,
    authority,
    PUBLISHER_MINT_AMOUNT,
  );

  return {
    provider,
    authority,
    publisher,
    gamer,
    pglProgram,
    registryProgram,
    storeProgram,
    paymentMint,
    pglConfigPda,
    registryConfigPda,
    storeConfigPda,
    authorizedSourceProgramPda,
    authorizedRegistryProgramPda,
    storeActorAuthorizedPda,
    registryAcceptedPaymentTokenPda,
    storeAcceptedPaymentTokenPda,
  };
}

export async function setupPeridotFixture(): Promise<BaseFixture> {
  if (!baseFixturePromise) {
    baseFixturePromise = initializeBaseFixture();
  }
  return baseFixturePromise;
}

export async function createRegisteredGame(
  base: BaseFixture,
  opts?: {
    gameId?: string;
    metadataUri?: string;
    publisher?: Keypair;
    mintToAmount?: number;
  },
): Promise<GameFixture> {
  const publisher = opts?.publisher ?? base.publisher;
  const gameId =
    opts?.gameId ??
    `pv-game-${Date.now()}-${Math.floor(Math.random() * 100000)}`;
  const metadataUri = opts?.metadataUri ?? `https://meta.peridot/${gameId}.json`;

  await maybeFundSigner(base.provider, publisher);

  const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
    base.provider.connection,
    base.authority,
    base.paymentMint,
    publisher.publicKey,
  );

  if ((opts?.mintToAmount ?? 0) > 0) {
    await mintTo(
      base.provider.connection,
      base.authority,
      base.paymentMint,
      publisherPaymentAta.address,
      base.authority,
      opts?.mintToAmount ?? 0,
    );
  }

  const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
    base.registryConfigPda,
  )) as any;

  const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
    base.provider.connection,
    base.authority,
    base.paymentMint,
    registryConfig.treasury,
  );

  const creatorStatePda = derivePda(
    [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
    base.pglProgram.programId,
  );

  let nextNonce = BigInt(0);
  if (await accountExists(base.provider.connection, creatorStatePda)) {
    const creatorState = (await base.pglProgram.account.creatorState.fetch(
      creatorStatePda,
    )) as any;
    nextNonce = BigInt(creatorState.nextNonce.toString());
  }

  const gamePda = derivePda(
    [
      Buffer.from("game"),
      publisher.publicKey.toBuffer(),
      u64LeBuffer(nextNonce),
    ],
    base.pglProgram.programId,
  );

  const registryGamePda = derivePda(
    [Buffer.from("registry_game"), gamePda.toBuffer()],
    base.registryProgram.programId,
  );

  const publishGrantPda = derivePda(
    [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
    base.registryProgram.programId,
  );

  if (!(await accountExists(base.provider.connection, publishGrantPda))) {
    await base.registryProgram.methods
      .createPublishGrant(null)
      .accounts({
        authority: base.authority.publicKey,
        config: base.registryConfigPda,
        publisher: publisher.publicKey,
        publishGrant: publishGrantPda,
        system_program: SystemProgram.programId,
      })
      .signers([publisher])
      .rpc();
  }

  const pglConfig = (await base.pglProgram.account.pglConfig.fetch(
    base.pglConfigPda,
  )) as any;

  const storeGameStoreConfigPda = derivePda(
    [Buffer.from("game_store_config"), gamePda.toBuffer()],
    base.storeProgram.programId,
  );

  await base.registryProgram.methods
    .createGameAndRegister(gameId, metadataUri, null, null)
    .accounts({
      publisher: publisher.publicKey,
      config: base.registryConfigPda,
      payment_mint: base.paymentMint,
      accepted_payment_token: base.registryAcceptedPaymentTokenPda,
      publisher_payment_account: publisherPaymentAta.address,
      treasury_payment_account: treasuryPaymentAta.address,
      registry_game: registryGamePda,
      game: gamePda,
      pglCreatorState: creatorStatePda,
      pglConfig: base.pglConfigPda,
      pglTreasury: pglConfig.treasury,
      pgl1_program: base.pglProgram.programId,
      storeProgram: base.storeProgram.programId,
      storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
      storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
      storeGameStoreConfig: storeGameStoreConfigPda,
      self_program: base.registryProgram.programId,
      token_program: TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    })
    .remainingAccounts([
      {
        pubkey: publishGrantPda,
        isWritable: false,
        isSigner: false,
      },
    ])
    .signers([publisher])
    .rpc();

  return {
    gameId,
    metadataUri,
    gamePda,
    creatorStatePda,
    registryGamePda,
    publisher,
  };
}

export async function configureStoreForGame(
  base: BaseFixture,
  game: GameFixture,
  opts?: {
    basePrice?: number;
    paymentMint?: PublicKey;
    active?: boolean;
  },
): Promise<StoreGameFixture> {
  const paymentMint = opts?.paymentMint ?? base.paymentMint;
  const basePrice = opts?.basePrice ?? DEFAULT_GAME_PRICE;
  const active = opts?.active ?? true;

  const storeAcceptedPaymentTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
    base.storeProgram.programId,
  );

  if (!(await accountExists(base.provider.connection, storeAcceptedPaymentTokenPda))) {
    await base.storeProgram.methods
      .addPaymentToken()
      .accounts({
        authority: base.authority.publicKey,
        store_config: base.storeConfigPda,
        mint: paymentMint,
        accepted_payment_token: storeAcceptedPaymentTokenPda,
        system_program: SystemProgram.programId,
      })
      .rpc();
  }

  const gameStoreConfigPda = derivePda(
    [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
    base.storeProgram.programId,
  );

  if (!(await accountExists(base.provider.connection, gameStoreConfigPda))) {
    await base.storeProgram.methods
      .initGameStoreConfig(active)
      .accounts({
        publisher: game.publisher.publicKey,
        authorized_source_program: base.authorizedSourceProgramPda,
        source_program: base.pglProgram.programId,
        authorized_registry_program: base.authorizedRegistryProgramPda,
        registry_program: base.registryProgram.programId,
        game: game.gamePda,
        registry_game: game.registryGamePda,
        game_store_config: gameStoreConfigPda,
        system_program: SystemProgram.programId,
      })
      .signers([game.publisher])
      .rpc();
  }

  const gamePaymentOptionPda = derivePda(
    [
      Buffer.from("game_payment_option"),
      game.gamePda.toBuffer(),
      paymentMint.toBuffer(),
    ],
    base.storeProgram.programId,
  );

  await base.storeProgram.methods
    .setGamePaymentOption(new anchor.BN(basePrice), active)
    .accounts({
      publisher: game.publisher.publicKey,
      authorized_source_program: base.authorizedSourceProgramPda,
      source_program: base.pglProgram.programId,
      authorized_registry_program: base.authorizedRegistryProgramPda,
      registry_program: base.registryProgram.programId,
      game: game.gamePda,
      registry_game: game.registryGamePda,
      game_store_config: gameStoreConfigPda,
      mint: paymentMint,
      accepted_payment_token: storeAcceptedPaymentTokenPda,
      game_payment_option: gamePaymentOptionPda,
      system_program: SystemProgram.programId,
    })
    .signers([game.publisher])
    .rpc();

  return {
    gameStoreConfigPda,
    gamePaymentOptionPda,
    paymentMint,
    basePrice,
  };
}

export async function buyGameForBuyer(
  base: BaseFixture,
  game: GameFixture,
  mintToken: PublicKey | null,
  opts?: {
    buyer?: Keypair;
    referrer?: PublicKey | null;
  },
): Promise<{ buyer: Keypair; purchaseReceiptPda: PublicKey }> {
  const buyer = opts?.buyer ?? base.gamer;
  const referrer = opts?.referrer ?? null;

  await maybeFundSigner(base.provider, buyer);

  const gameStoreConfigPda = derivePda(
    [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
    base.storeProgram.programId,
  );
  const purchaseReceiptPda = derivePda(
    [
      Buffer.from("purchase_receipt"),
      buyer.publicKey.toBuffer(),
      game.gamePda.toBuffer(),
    ],
    base.storeProgram.programId,
  );
  const licensePda = derivePda(
    [Buffer.from("license"), buyer.publicKey.toBuffer(), game.gamePda.toBuffer()],
    base.pglProgram.programId,
  );

  if (mintToken === null) {
    // ── Free game path ──────────────────────────────────────────────
    return base.storeProgram.methods
      .buyGame(null, referrer)
      .accounts({
        buyer: buyer.publicKey,
        store_config: base.storeConfigPda,
        authorized_source_program: base.authorizedSourceProgramPda,
        source_program: base.pglProgram.programId,
        authorized_registry_program: base.authorizedRegistryProgramPda,
        registry_program: base.registryProgram.programId,
        game: game.gamePda,
        registry_game: game.registryGamePda,
        game_store_config: gameStoreConfigPda,
        payment_mint: null,
        accepted_payment_token: null,
        game_payment_option: null,
        buyer_payment_account: null,
        publisher_payment_account: null,
        treasury_payment_account: null,
        referrer_payment_account: null,
        store_actor: base.authority.publicKey,
        authorized_actor: base.storeActorAuthorizedPda,
        pgl1_program: base.pglProgram.programId,
        license: licensePda,
        purchase_receipt: purchaseReceiptPda,
        token_program: TOKEN_PROGRAM_ID,
        system_program: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc()
      .then(() => ({ buyer, purchaseReceiptPda }));
  }

  // ── Paid game path ────────────────────────────────────────────────
  const paymentMint = mintToken;

  const storeAcceptedPaymentTokenPda = derivePda(
    [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
    base.storeProgram.programId,
  );
  const gamePaymentOptionPda = derivePda(
    [
      Buffer.from("game_payment_option"),
      game.gamePda.toBuffer(),
      paymentMint.toBuffer(),
    ],
    base.storeProgram.programId,
  );

  // Fetch on-chain price and discount config
  const [gpoAccount, storeCfgAccount] = await Promise.all([
    base.storeProgram.account.gamePaymentOption.fetch(gamePaymentOptionPda) as any,
    base.storeProgram.account.gameStoreConfig.fetch(gameStoreConfigPda) as any,
  ]);

  const basePrice = gpoAccount.basePrice.toNumber();
  let finalPrice = basePrice;

  if (storeCfgAccount.discountBps !== null && storeCfgAccount.discountBps !== undefined) {
    const bps = storeCfgAccount.discountBps;
    const now = Math.floor(Date.now() / 1000);
    const startsAt = storeCfgAccount.discountStartsAt?.toNumber?.() ?? null;
    const expiresAt = storeCfgAccount.discountExpiresAt?.toNumber?.() ?? null;

    if (
      (startsAt === null || now >= startsAt) &&
      (expiresAt === null || now <= expiresAt)
    ) {
      finalPrice = basePrice - Math.floor(basePrice * bps / 10_000);
    }
  }

  // Token accounts
  const buyerPaymentAta = await getOrCreateAssociatedTokenAccount(
    base.provider.connection,
    base.authority,
    paymentMint,
    buyer.publicKey,
  );
  const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
    base.provider.connection,
    base.authority,
    paymentMint,
    game.publisher.publicKey,
  );

  const storeConfig = (await base.storeProgram.account.storeConfig.fetch(
    base.storeConfigPda,
  )) as any;
  const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
    base.provider.connection,
    base.authority,
    paymentMint,
    storeConfig.treasury,
  );

  const referrerPaymentAta =
    referrer === null
      ? null
      : await getOrCreateAssociatedTokenAccount(
          base.provider.connection,
          base.authority,
          paymentMint,
          referrer,
        );

  await mintTo(
    base.provider.connection,
    base.authority,
    paymentMint,
    buyerPaymentAta.address,
    base.authority,
    finalPrice,
  );

  await base.storeProgram.methods
    .buyGame(mintToken, referrer)
    .accounts({
      buyer: buyer.publicKey,
      store_config: base.storeConfigPda,
      authorized_source_program: base.authorizedSourceProgramPda,
      source_program: base.pglProgram.programId,
      authorized_registry_program: base.authorizedRegistryProgramPda,
      registry_program: base.registryProgram.programId,
      game: game.gamePda,
      registry_game: game.registryGamePda,
      game_store_config: gameStoreConfigPda,
      paymentMint,
      accepted_payment_token: storeAcceptedPaymentTokenPda,
      game_payment_option: gamePaymentOptionPda,
      buyer_payment_account: buyerPaymentAta.address,
      publisher_payment_account: publisherPaymentAta.address,
      treasury_payment_account: treasuryPaymentAta.address,
      referrer_payment_account: referrerPaymentAta?.address ?? null,
      store_actor: base.authority.publicKey,
      authorized_actor: base.storeActorAuthorizedPda,
      pgl1_program: base.pglProgram.programId,
      license: licensePda,
      purchase_receipt: purchaseReceiptPda,
      token_program: TOKEN_PROGRAM_ID,
      system_program: SystemProgram.programId,
    })
    .signers([buyer])
    .rpc();

  return { buyer, purchaseReceiptPda };
}

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

export const PGL1_PROGRAM_ID = new PublicKey("DzDbFZXZsmFFv1mMFimLaBjAQi7Z5gUaQ61qcDuR6Kor");
export const REGISTRY_PROGRAM_ID = new PublicKey("DCYPxPtnVeBgy56SYMT6GPBMJp8NJNLmE46QfHYqCgGL");
export const STORE_PROGRAM_ID = new PublicKey("6gTd8TQ9NiC7yxBfGWBzH1aWdk77fg779nUJhYTrEsPd");

export const DEFAULT_PLATFORM_FEE_BPS = 1_000;
export const DEFAULT_REFERRAL_BPS = 200;
export const DEFAULT_MAX_REFERRAL_BPS = 5_000;
export const DEFAULT_REGISTRY_FEE = 1_000;
export const DEFAULT_GAME_PRICE = 20_000_000;
export const PAYMENT_DECIMALS = 6;
export const PUBLISHER_MINT_AMOUNT = 1_000_000_000;

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
  paymentMint: PublicKey;
  basePrice: number;
};

type NodeWallet = anchor.Wallet & { payer: Keypair };

export type BaseFixture = {
  provider: anchor.AnchorProvider;
  authority: Keypair;
  publisher: Keypair;
  gamer: Keypair;
  pglProgram: any;
  registryProgram: any;
  storeProgram: any;
  paymentMint: PublicKey;
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
  const storeIdl = require("../../target/idl/peridotvault_store.json");

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
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, registryConfigPda))) {
    await registryProgram.methods
      .initializeRegistry(authority.publicKey)
      .accounts({
        authority: authority.publicKey,
        config: registryConfigPda,
        systemProgram: SystemProgram.programId,
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
        storeConfig: storeConfigPda,
        systemProgram: SystemProgram.programId,
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
        acceptedPaymentToken: registryAcceptedPaymentTokenPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, storeAcceptedPaymentTokenPda))) {
    await storeProgram.methods
      .addPaymentToken()
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        mint: paymentMint,
        acceptedPaymentToken: storeAcceptedPaymentTokenPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  const authorizedSourceProgramPda = derivePda(
    [Buffer.from("authorized_source_program"), pglProgram.programId.toBuffer()],
    storeProgram.programId,
  );
  const authorizedRegistryProgramPda = derivePda(
    [Buffer.from("authorized_registry_program"), registryProgram.programId.toBuffer()],
    storeProgram.programId,
  );

  if (!(await accountExists(provider.connection, authorizedSourceProgramPda))) {
    await storeProgram.methods
      .addAuthorizedSourceProgram()
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        programId: pglProgram.programId,
        authorizedSourceProgram: authorizedSourceProgramPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, authorizedRegistryProgramPda))) {
    await storeProgram.methods
      .addAuthorizedRegistryProgram()
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        programId: registryProgram.programId,
        authorizedRegistryProgram: authorizedRegistryProgramPda,
        systemProgram: SystemProgram.programId,
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
        authorizedActor: storeActorAuthorizedPda,
        systemProgram: SystemProgram.programId,
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

  await base.registryProgram.methods
    .createPublishGrant(null)
    .accounts({
      authority: base.authority.publicKey,
      config: base.registryConfigPda,
      publisher: publisher.publicKey,
      publishGrant: publishGrantPda,
      systemProgram: SystemProgram.programId,
    })
    .signers([publisher])
    .rpc();

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
      paymentMint: base.paymentMint,
      acceptedPaymentToken: base.registryAcceptedPaymentTokenPda,
      publisherPaymentAccount: publisherPaymentAta.address,
      treasuryPaymentAccount: treasuryPaymentAta.address,
      registryGame: registryGamePda,
      game: gamePda,
      pglCreatorState: creatorStatePda,
      pglConfig: base.pglConfigPda,
      pglTreasury: pglConfig.treasury,
      pgl1Program: base.pglProgram.programId,
      storeProgram: base.storeProgram.programId,
      storeAuthorizedSourceProgram: base.authorizedSourceProgramPda,
      storeAuthorizedRegistryProgram: base.authorizedRegistryProgramPda,
      storeGameStoreConfig: storeGameStoreConfigPda,
      selfProgram: base.registryProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
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
        storeConfig: base.storeConfigPda,
        mint: paymentMint,
        acceptedPaymentToken: storeAcceptedPaymentTokenPda,
        systemProgram: SystemProgram.programId,
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
        authorizedSourceProgram: base.authorizedSourceProgramPda,
        sourceProgram: base.pglProgram.programId,
        authorizedRegistryProgram: base.authorizedRegistryProgramPda,
        registryProgram: base.registryProgram.programId,
        game: game.gamePda,
        registryGame: game.registryGamePda,
        gameStoreConfig: gameStoreConfigPda,
        systemProgram: SystemProgram.programId,
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
      authorizedSourceProgram: base.authorizedSourceProgramPda,
      sourceProgram: base.pglProgram.programId,
      authorizedRegistryProgram: base.authorizedRegistryProgramPda,
      registryProgram: base.registryProgram.programId,
      game: game.gamePda,
      registryGame: game.registryGamePda,
      gameStoreConfig: gameStoreConfigPda,
      mint: paymentMint,
      acceptedPaymentToken: storeAcceptedPaymentTokenPda,
      gamePaymentOption: gamePaymentOptionPda,
      systemProgram: SystemProgram.programId,
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
  paidAmount: number,
  opts?: {
    buyer?: Keypair;
    paymentMint?: PublicKey;
    referrer?: PublicKey | null;
  },
): Promise<{ buyer: Keypair; purchaseReceiptPda: PublicKey }> {
  const buyer = opts?.buyer ?? base.gamer;
  const paymentMint = opts?.paymentMint ?? base.paymentMint;
  const referrer = opts?.referrer ?? null;

  await maybeFundSigner(base.provider, buyer);

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
    paidAmount,
  );

  const gameStoreConfigPda = derivePda(
    [Buffer.from("game_store_config"), game.gamePda.toBuffer()],
    base.storeProgram.programId,
  );
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

  let builder = base.storeProgram.methods
    .buyGame(new anchor.BN(paidAmount), referrer)
    .accounts({
      buyer: buyer.publicKey,
      storeConfig: base.storeConfigPda,
      authorizedSourceProgram: base.authorizedSourceProgramPda,
      sourceProgram: base.pglProgram.programId,
      authorizedRegistryProgram: base.authorizedRegistryProgramPda,
      registryProgram: base.registryProgram.programId,
      game: game.gamePda,
      registryGame: game.registryGamePda,
      gameStoreConfig: gameStoreConfigPda,
      paymentMint,
      acceptedPaymentToken: storeAcceptedPaymentTokenPda,
      gamePaymentOption: gamePaymentOptionPda,
      buyerPaymentAccount: buyerPaymentAta.address,
      publisherPaymentAccount: publisherPaymentAta.address,
      treasuryPaymentAccount: treasuryPaymentAta.address,
      referrerPaymentAccount: referrerPaymentAta?.address ?? null,
      storeActor: base.authority.publicKey,
      authorizedActor: base.storeActorAuthorizedPda,
      pgl1Program: base.pglProgram.programId,
      license: licensePda,
      purchaseReceipt: purchaseReceiptPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([buyer]);

  await builder.rpc();

  return { buyer, purchaseReceiptPda };
}

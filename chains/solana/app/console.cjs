const anchor = require("@coral-xyz/anchor");
const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} = require("@solana/spl-token");
const { Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const { createHash } = require("crypto");
const fs = require("fs");
const os = require("os");
const path = require("path");
const readline = require("readline/promises");
const { stdin: input, stdout: output } = require("process");

const pgc1Idl = require("../target/idl/pgc1.json");
const registryIdl = require("../target/idl/registry.json");
const storeIdl = require("../target/idl/game_store.json");

// Seeds from programs
const CONFIG_SEED = Buffer.from("config");
const GAME_SEED = Buffer.from("game");
const LICENSE_SEED = Buffer.from("license");
const MINTER_SEED = Buffer.from("minter");
const PRICE_SEED = Buffer.from("price");
const BALANCE_SEED = Buffer.from("balance");

const STATUS_ACTIVE = true;
const PAYMENT_DECIMALS = 9; // SOL

function shortAddress(pubkey) {
  const value = pubkey.toBase58();
  return `${value.slice(0, 6)}...${value.slice(-6)}`;
}

function loadProvider() {
  const providerUrl = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
  const walletPath =
    process.env.ANCHOR_WALLET || path.join(os.homedir(), ".config/solana/id.json");
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
  const connection = new anchor.web3.Connection(providerUrl, "confirmed");
  return new anchor.AnchorProvider(connection, wallet, anchor.AnchorProvider.defaultOptions());
}

function derivePda(seeds, programId) {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

async function accountExists(connection, address) {
  return (await connection.getAccountInfo(address)) !== null;
}

function makePrograms(provider) {
  const pgc1Program = new anchor.Program(pgc1Idl, provider);
  const registryProgram = new anchor.Program(registryIdl, provider);
  const storeProgram = new anchor.Program(storeIdl, provider);
  return { pgc1Program, registryProgram, storeProgram };
}

function deriveProgramAccounts(ctx, gameId) {
  // PGC1 PDAs
  const pgcGamePda = derivePda([GAME_SEED, Buffer.from(gameId)], ctx.pgc1Program.programId);
  
  // Registry PDAs
  const registryGamePda = derivePda([GAME_SEED, Buffer.from(gameId)], ctx.registryProgram.programId);
  const registryConfigPda = derivePda([CONFIG_SEED], ctx.registryProgram.programId);

  // Store PDAs
  const storeConfigPda = derivePda([CONFIG_SEED], ctx.storeProgram.programId);
  const pricePda = derivePda([PRICE_SEED, pgcGamePda.toBuffer()], ctx.storeProgram.programId);

  // PGC1 Minter PDA (authorizing the Store to mint for this game)
  const pgcMinterAccount = derivePda([MINTER_SEED, pgcGamePda.toBuffer(), storeConfigPda.toBuffer()], ctx.pgc1Program.programId);

  return {
    pgcGamePda,
    registryGamePda,
    registryConfigPda,
    storeConfigPda,
    pricePda,
    pgcMinterAccount,
  };
}

async function getCatalog(ctx) {
  const registrations = await ctx.registryProgram.account.registryGameAccount.all();
  const catalog = [];
  for (const reg of registrations) {
    const game = reg.account;
    const accounts = deriveProgramAccounts(ctx, game.gameId);
    let price = null;
    let currency = null;
    try {
      const priceAccount = await ctx.storeProgram.account.priceAccount.fetch(accounts.pricePda);
      price = priceAccount.price;
      currency = priceAccount.currency;
    } catch (e) {}
    
    catalog.push({
      gameId: game.gameId,
      publisher: game.publisher,
      active: game.active,
      price,
      currency,
    });
  }
  return catalog;
}

async function createGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id: ")).trim();
  if (!gameId) return console.log("game id is required");
  const price = (await rl.question("enter price in SOL (default 0.1): ")).trim() || "0.1";
  const lamports = new anchor.BN(parseFloat(price) * 1e9);

  const accounts = deriveProgramAccounts(ctx, gameId);
  
  console.log(`Starting Atomic Setup for ${gameId} via PGC1...`);
  const tx = await ctx.pgc1Program.methods
    .createGame(
      gameId, 
      `https://meta.peridot/${gameId}`, 
      accounts.storeConfigPda, // Authorize Store to mint licenses immediately
      lamports, 
      SystemProgram.programId
    )
    .accounts({
      publisher: ctx.user.publicKey,
      gameAccount: accounts.pgcGamePda,
      initialMinterAccount: accounts.pgcMinterAccount,
      registryProgram: ctx.registryProgram.programId,
      storeProgram: ctx.storeProgram.programId,
      registryGame: accounts.registryGamePda,
      priceAccount: accounts.pricePda,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(`Successfully created, registered, and priced ${gameId}!`);
  console.log(`Transaction Signature: ${tx}`);
}

async function buyGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id to buy: ")).trim();
  if (!gameId) return;

  const accounts = deriveProgramAccounts(ctx, gameId);
  const licensePda = derivePda([LICENSE_SEED, ctx.user.publicKey.toBuffer(), accounts.pgcGamePda.toBuffer()], ctx.pgc1Program.programId);
  
  const storeConfig = await ctx.storeProgram.account.storeConfig.fetch(accounts.storeConfigPda);
  const priceAccount = await ctx.storeProgram.account.priceAccount.fetch(accounts.pricePda);
  const pgcGame = await ctx.pgc1Program.account.pgcGameAccount.fetch(accounts.pgcGamePda);

  const publisherBalancePda = derivePda([BALANCE_SEED, pgcGame.publisher.toBuffer(), priceAccount.currency.toBuffer()], ctx.storeProgram.programId);

  console.log(`Buying ${gameId} for ${priceAccount.price.toNumber() / 1e9} SOL...`);
  
  const tx = await ctx.storeProgram.methods
    .buyGame()
    .accounts({
      buyer: ctx.user.publicKey,
      storeConfig: accounts.storeConfigPda,
      treasury: storeConfig.treasury,
      pgcGameState: accounts.pgcGamePda,
      priceAccount: accounts.pricePda,
      affiliateAccount: null,
      affiliate: null,
      publisherBalance: publisherBalancePda,
      pgcMinterAccount: accounts.pgcMinterAccount,
      pgcLicenseAccount: licensePda,
      pgc1Program: ctx.pgc1Program.programId,
      systemProgram: SystemProgram.programId,
    })
    .signers([ctx.user])
    .rpc();

  console.log(`Bought ${gameId}, tx: ${tx}`);
}

async function bootstrap() {
  const provider = loadProvider();
  anchor.setProvider(provider);
  const programs = makePrograms(provider);
  const user = provider.wallet.payer;

  const regConfigPda = derivePda([CONFIG_SEED], programs.registryProgram.programId);
  const storeConfigPda = derivePda([CONFIG_SEED], programs.storeProgram.programId);

  if (!(await accountExists(provider.connection, regConfigPda))) {
    console.log("Initializing Registry...");
    await programs.registryProgram.methods.initialize(user.publicKey).accounts({ payer: user.publicKey, registryConfig: regConfigPda, systemProgram: SystemProgram.programId }).rpc();
  }
  
  if (!(await accountExists(provider.connection, storeConfigPda))) {
    console.log("Initializing Game Store...");
    await programs.storeProgram.methods.initialize(user.publicKey, user.publicKey, 100).accounts({ payer: user.publicKey, storeConfig: storeConfigPda, systemProgram: SystemProgram.programId }).rpc();
  }

  return { ...programs, provider, user, regConfigPda, storeConfigPda };
}

async function main() {
  const rl = readline.createInterface({ input, output });
  try {
    const ctx = await bootstrap();
    while (true) {
      console.log("\n--- PERIDOT CONSOLE ---");
      console.log("1. List Games");
      console.log("2. Buy Game");
      console.log("3. Create Game (Unified)");
      console.log("0. Exit");
      const choice = (await rl.question("choose: ")).trim();
      if (choice === "0") break;
      if (choice === "1") {
        const catalog = await getCatalog(ctx);
        catalog.forEach(g => console.log(`- ${g.gameId}: ${g.price ? (g.price.toNumber() / 1e9).toFixed(2) + " SOL" : "N/A"} [Publisher: ${shortAddress(g.publisher)}]`));
      } else if (choice === "2") await buyGameFlow(ctx, rl);
      else if (choice === "3") await createGameFlow(ctx, rl);
    }
  } finally { rl.close(); }
}

main().catch(console.error);

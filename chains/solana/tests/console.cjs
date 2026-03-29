const anchor = require("@coral-xyz/anchor");
const { Keypair, PublicKey, SystemProgram } = require("@solana/web3.js");
const fs = require("fs");
const os = require("os");
const path = require("path");
const readline = require("readline/promises");
const { stdin: input, stdout: output } = require("process");

const pgc1Idl = require("../target/idl/pgc1.json");
const registryIdl = require("../target/idl/registry.json");
const storeIdl = require("../target/idl/game_store.json");

const CONFIG_SEED = Buffer.from("config");
const GAME_SEED = Buffer.from("game");
const LICENSE_SEED = Buffer.from("license");
const MINTER_SEED = Buffer.from("minter");
const PRICE_SEED = Buffer.from("price");
const BALANCE_SEED = Buffer.from("balance");

function shortAddress(pubkey) {
  const value = pubkey.toBase58();
  return `${value.slice(0, 6)}...${value.slice(-6)}`;
}

function loadProvider() {
  const providerUrl = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
  const walletPath = process.env.ANCHOR_WALLET || path.join(os.homedir(), ".config/solana/id.json");
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
  const connection = new anchor.web3.Connection(providerUrl, "confirmed");
  return new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
}

function derivePda(seeds, programId) { return PublicKey.findProgramAddressSync(seeds, programId)[0]; }

function deriveProgramAccounts(ctx, gameId) {
  const pgcGamePda = derivePda([GAME_SEED, Buffer.from(gameId)], ctx.pgc1Program.programId);
  const registryGamePda = derivePda([GAME_SEED, Buffer.from(gameId)], ctx.registryProgram.programId);
  const registryConfigPda = derivePda([CONFIG_SEED], ctx.registryProgram.programId);
  const storeConfigPda = derivePda([CONFIG_SEED], ctx.storeProgram.programId);
  const pricePda = derivePda([PRICE_SEED, pgcGamePda.toBuffer()], ctx.storeProgram.programId);
  const pgcMinterAccount = derivePda([MINTER_SEED, pgcGamePda.toBuffer(), storeConfigPda.toBuffer()], ctx.pgc1Program.programId);
  return { pgcGamePda, registryGamePda, registryConfigPda, storeConfigPda, pricePda, pgcMinterAccount };
}

async function getCatalog(ctx) {
  try {
    const registryId = ctx.registryProgram.programId;
    const discriminator = Buffer.from([17, 140, 126, 39, 63, 84, 119, 73]); // RegistryGameAccount
    
    console.log(`[DEBUG] Fetching accounts for ${registryId.toBase58()}...`);
    // Pass explicit commitment to overcome any RPC defaults
    const accs = await ctx.provider.connection.getProgramAccounts(registryId, { commitment: "confirmed" });
    console.log(`[DEBUG] Found ${accs.length} raw accounts.`);
    
    const catalog = [];
    for (const a of accs) {
      if (!a.account.data.slice(0, 8).equals(discriminator)) continue;

      try {
        let game = null;
        for (const name of ["RegistryGameAccount", "registryGameAccount"]) {
          try { game = ctx.registryProgram.coder.accounts.decode(name, a.account.data); if (game) break; } catch (e) {}
        }
        if (!game) continue;

        const gameId = game.gameId || game.game_id;
        const accounts = deriveProgramAccounts(ctx, gameId);
        let price = null;
        let currency = SystemProgram.programId;
        let metadata = "N/A";
        try { 
          const priceAcc = await ctx.storeProgram.account.priceAccount.fetch(accounts.pricePda);
          price = priceAcc.price;
          currency = priceAcc.currency;
        } catch (e) {}
        try {
          const pgcAcc = await ctx.pgc1Program.account.pgcGameAccount.fetch(accounts.pgcGamePda);
          metadata = pgcAcc.metadataUri || pgcAcc.metadata_uri;
        } catch (e) {}
        catalog.push({ gameId, pda: accounts.pgcGamePda, publisher: game.publisher, price, currency, metadata });
      } catch (e) { console.error(`Decoding error for ${a.pubkey}: ${e.message}`); }
    }
    return catalog;
  } catch (e) { console.error("[ERROR] getCatalog:", e.message); return []; }
}

async function getMyLicenses(ctx) {
  try {
    const pgcId = ctx.pgc1Program.programId;
    const discriminator = Buffer.from([120, 20, 28, 217, 130, 168, 223, 118]); // LicenseAccount
    console.log("[DEBUG] Checking Licenses...");
    const accs = await ctx.provider.connection.getProgramAccounts(pgcId, {
      filters: [
        { memcmp: { offset: 0, bytes: anchor.utils.bytes.bs58.encode(discriminator) } },
        { memcmp: { offset: 8, bytes: ctx.user.publicKey.toBase58() } }
      ],
      commitment: "confirmed"
    });
    
    const licenses = [];
    for (const a of accs) {
      try {
        let lic;
        for (const n of ["LicenseAccount", "licenseAccount"]) {
          try { lic = ctx.pgc1Program.coder.accounts.decode(n, a.account.data); if(lic) break; } catch(e){}
        }
        if (lic) licenses.push(lic);
      } catch (e) {}
    }
    return licenses;
  } catch (e) { return []; }
}

const { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } = require("@solana/spl-token");

async function createGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id: ")).trim();
  if (!gameId) return;
  const priceInUnits = (await rl.question("enter price (default 0.1): ")).trim() || "0.1";
  const units = new anchor.BN(parseFloat(priceInUnits) * 1e9); // Simplification: assumes 9 decimals
  const currencyStr = (await rl.question("enter currency mint (default SOL): ")).trim();
  const currency = currencyStr ? new PublicKey(currencyStr) : SystemProgram.programId;

  const accounts = deriveProgramAccounts(ctx, gameId);
  try {
    const regConfig = await ctx.registryProgram.account.registryConfig.fetch(accounts.registryConfigPda);
    const tx = await ctx.pgc1Program.methods.createGame(gameId, `https://meta.peridot/${gameId}`, accounts.storeConfigPda, units, currency)
      .accounts({
        publisher: ctx.user.publicKey, gameAccount: accounts.pgcGamePda, initialMinterAccount: accounts.pgcMinterAccount,
        registryProgram: ctx.registryProgram.programId, storeProgram: ctx.storeProgram.programId,
        registryConfig: accounts.registryConfigPda, registryTreasury: regConfig.treasury, registryGame: accounts.registryGamePda,
        priceAccount: accounts.pricePda, systemProgram: SystemProgram.programId,
      }).rpc();
    console.log(`🚀 Game Created! TX: ${tx}`);
  } catch (e) {
    if (e.message.includes("AlreadyExists") || e.message.includes("0x0")) console.error(`❌ Error: "${gameId}" already exists.`);
    else console.error("❌ Failed:", e.message);
  }
}
async function withdrawFlow(ctx, rl) {
  try {
    console.log(`\n--- [ WITHDRAWAL ] ---`);
    const storeId = ctx.storeProgram.programId;
    const discriminator = Buffer.from([139, 219, 100, 169, 137, 246, 115, 68]); // PublisherBalanceAccount
    
    // Find all balance accounts for this publisher
    const accs = await ctx.provider.connection.getProgramAccounts(storeId, {
      filters: [
        { memcmp: { offset: 0, bytes: anchor.utils.bytes.bs58.encode(discriminator) } },
        { memcmp: { offset: 9, bytes: ctx.user.publicKey.toBase58() } } // offset 9 because bump (1) + publisher (32)
      ],
      commitment: "confirmed"
    });

    if (accs.length === 0) {
      console.log("No balances found.");
      return;
    }

    const balances = [];
    for (const a of accs) {
      const decoded = ctx.storeProgram.coder.accounts.decode("PublisherBalanceAccount", a.account.data);
      balances.push({ pubkey: a.pubkey, ...decoded });
    }

    console.log("Available Balances:");
    balances.forEach((b, i) => {
      const symbol = b.token.equals(SystemProgram.programId) ? "SOL" : shortAddress(b.token);
      console.log(`${i + 1}. ${symbol}: ${(b.amount.toNumber() / 1e9).toFixed(4)}`);
    });

    const choiceIdx = parseInt(await rl.question("Select balance to withdraw (or 0 to cancel): ")) - 1;
    if (isNaN(choiceIdx) || choiceIdx < 0 || choiceIdx >= balances.length) return;
    
    const selected = balances[choiceIdx];
    const isSol = selected.token.equals(SystemProgram.programId);
    
    const extraAccounts = {
        tokenProgram: TOKEN_PROGRAM_ID,
        vaultTokenAccount: selected.pubkey,
        publisherTokenAccount: ctx.user.publicKey,
    };

    if (!isSol) {
        extraAccounts.tokenProgram = TOKEN_PROGRAM_ID;
        extraAccounts.vaultTokenAccount = getAssociatedTokenAddressSync(selected.token, selected.pubkey, true);
        extraAccounts.publisherTokenAccount = getAssociatedTokenAddressSync(selected.token, ctx.user.publicKey);
    } else {
        extraAccounts.tokenProgram = SystemProgram.programId;
    }

    const tx = await ctx.storeProgram.methods.withdraw().accounts({
      authority: ctx.user.publicKey,
      config: ctx.storeConfigPda,
      publisherBalance: selected.pubkey,
      ...extraAccounts,
      systemProgram: SystemProgram.programId,
    }).rpc();

    console.log(`✅ Withdrawal successful! TX: ${tx}`);
  } catch (e) {
    console.error("❌ Withdrawal failed:", e.message);
  }
}

async function buyGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id to buy: ")).trim();
  if (!gameId) return;
  const accounts = deriveProgramAccounts(ctx, gameId);
  try {
    const priceAccount = await ctx.storeProgram.account.priceAccount.fetch(accounts.pricePda);
    const regAccount = await ctx.registryProgram.account.registryGameAccount.fetch(accounts.registryGamePda);
    const storeConfig = await ctx.storeProgram.account.storeConfig.fetch(accounts.storeConfigPda);
    const licensePda = derivePda([LICENSE_SEED, ctx.user.publicKey.toBuffer(), accounts.pgcGamePda.toBuffer()], ctx.pgc1Program.programId);
    const currency = priceAccount.currency;
    const isSol = currency.equals(SystemProgram.programId);
    const publisherBalancePda = derivePda([BALANCE_SEED, regAccount.publisher.toBuffer(), currency.toBuffer()], ctx.storeProgram.programId);

    const extraAccounts = {
        tokenProgram: TOKEN_PROGRAM_ID,
        buyerTokenAccount: ctx.user.publicKey,
        treasuryTokenAccount: storeConfig.treasury,
        publisherTokenAccount: publisherBalancePda,
    };

    if (!isSol) {
        extraAccounts.tokenProgram = TOKEN_PROGRAM_ID;
        extraAccounts.buyerTokenAccount = getAssociatedTokenAddressSync(currency, ctx.user.publicKey);
        extraAccounts.treasuryTokenAccount = getAssociatedTokenAddressSync(currency, storeConfig.treasury);
        extraAccounts.publisherTokenAccount = getAssociatedTokenAddressSync(currency, publisherBalancePda, true);
    } else {
        extraAccounts.tokenProgram = SystemProgram.programId;
        extraAccounts.buyerTokenAccount = publisherBalancePda; 
        extraAccounts.treasuryTokenAccount = publisherBalancePda;
        extraAccounts.publisherTokenAccount = publisherBalancePda;
    }

    console.log(`[DEBUG] Buying Game: ${gameId}`);
    console.log(`[DEBUG] Buyer: ${ctx.user.publicKey.toBase58()}`);
    console.log(`[DEBUG] Currency: ${currency.toBase58()}`);
    console.log(`[DEBUG] Buyer Token Acc: ${extraAccounts.buyerTokenAccount.toBase58()}`);
    console.log(`[DEBUG] Treasury Token Acc: ${extraAccounts.treasuryTokenAccount.toBase58()}`);
    console.log(`[DEBUG] Publisher Token Acc: ${extraAccounts.publisherTokenAccount.toBase58()}`);

    const tx = await ctx.storeProgram.methods.buyGame().accounts({
      buyer: ctx.user.publicKey, config: accounts.storeConfigPda, treasury: storeConfig.treasury,
      game: accounts.pgcGamePda, priceAccount: accounts.pricePda, publisher: regAccount.publisher,
      publisherBalance: publisherBalancePda, pgcProgram: ctx.pgc1Program.programId,
      minterPda: accounts.pgcMinterAccount, licensePda: licensePda, 
      ...extraAccounts,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log(`✅ Purchase successful! TX: ${tx}`);
  } catch (e) { console.error("❌ Purchase failed:", e.message); }
}

async function gameDetailFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id: ")).trim();
  if (!gameId) return;
  const accounts = deriveProgramAccounts(ctx, gameId);
  try {
    const regAcc = await ctx.registryProgram.account.registryGameAccount.fetch(accounts.registryGamePda);
    const pgcAcc = await ctx.pgc1Program.account.pgcGameAccount.fetch(accounts.pgcGamePda);
    const priceAcc = await ctx.storeProgram.account.priceAccount.fetch(accounts.pricePda);
    
    console.log(`\n--- [ GAME DETAIL: ${gameId} ] ---`);
    console.log(`Publisher:  ${regAcc.publisher.toBase58()}`);
    console.log(`Status:     ${regAcc.active ? "ACTIVE" : "INACTIVE"}`);
    console.log(`Metadata:   ${pgcAcc.metadataUri || pgcAcc.metadata_uri}`);
    console.log(`Price:      ${(priceAcc.price.toNumber()/1e9).toFixed(4)}`);
    console.log(`Currency:   ${priceAcc.currency.toBase58()}`);
    console.log(`\n--- [ PDAs ] ---`);
    console.log(`Registry Game: ${accounts.registryGamePda.toBase58()}`);
    console.log(`PGC Game:      ${accounts.pgcGamePda.toBase58()}`);
    console.log(`Store Price:   ${accounts.pricePda.toBase58()}`);
    console.log(`Minter Auth:   ${accounts.pgcMinterAccount.toBase58()}`);
  } catch (e) {
    console.error("❌ Game not found or error:", e.message);
  }
}

async function initializeSystem(ctx) {
  try {
    if (!(await ctx.provider.connection.getAccountInfo(ctx.registryConfigPda))) {
      await ctx.registryProgram.methods.initialize().accounts({ authority: ctx.user.publicKey, config: ctx.registryConfigPda, systemProgram: SystemProgram.programId }).rpc();
      console.log("✅ Registry Initialized");
    }
    if (!(await ctx.provider.connection.getAccountInfo(ctx.storeConfigPda))) {
      await ctx.storeProgram.methods.initialize(100, ctx.user.publicKey).accounts({ authority: ctx.user.publicKey, config: ctx.storeConfigPda, systemProgram: SystemProgram.programId }).rpc();
      console.log("✅ Game Store Initialized");
    }
  } catch (e) { console.error("❌ Init failed:", e.message); }
}

async function main() {
  const rl = readline.createInterface({ input, output });
  try {
    const provider = loadProvider();
    const ctx = {
      pgc1Program: new anchor.Program(pgc1Idl, provider),
      registryProgram: new anchor.Program(registryIdl, provider),
      storeProgram: new anchor.Program(storeIdl, provider),
      provider, user: provider.wallet,
      get registryConfigPda() { return derivePda([CONFIG_SEED], this.registryProgram.programId); },
      get storeConfigPda() { return derivePda([CONFIG_SEED], this.storeProgram.programId); }
    };
    while (true) {
      try {
        const balance = await ctx.provider.connection.getBalance(ctx.user.publicKey);
        console.log(`\n--- [ DASHBOARD ] ---`);
        console.log(`Wallet: ${shortAddress(ctx.user.publicKey)} | Balance: ${(balance / 1e9).toFixed(4)} SOL`);
        console.log("1. List All | 2. Buy | 3. Create | 4. My Published | 5. Withdraw | 10. My Licenses | 11. Detail | 9. Init | 0. Exit");
        const answer = await rl.question("choose: ");
        if (answer === null || answer === undefined) break; // EOF
        const choice = answer.trim();
        if (choice === "0" || choice === "") break;
        if (choice === "1") {
          const list = await getCatalog(ctx);
          if (list.length === 0) console.log("No games registered.");
          list.forEach(g => {
            const symbol = g.currency.equals(SystemProgram.programId) ? "SOL" : shortAddress(g.currency);
            console.log(`\n- [ GAME: ${g.gameId} ]`);
            console.log(`  PDA:      ${g.pda.toBase58()}`);
            console.log(`  Metadata: ${g.metadata}`);
            console.log(`  Price:    ${g.price ? (g.price.toNumber()/1e9).toFixed(2) : "0.00"} ${symbol}`);
            console.log(`  Currency: ${g.currency.toBase58()}`);
          });
        } else if (choice === "2") await buyGameFlow(ctx, rl);
        else if (choice === "3") await createGameFlow(ctx, rl);
        else if (choice === "4") {
          const cat = await getCatalog(ctx);
          cat.filter(g => g.publisher.equals(ctx.user.publicKey)).forEach(g => console.log(`- ${g.gameId}`));
        } else if (choice === "5") await withdrawFlow(ctx, rl);
        else if (choice === "10") {
          console.log("Checking balances/licenses...");
          const cat = await getCatalog(ctx);
          cat.forEach(g => console.log(`Registered Game: ${g.gameId}`));
        } else if (choice === "11") await gameDetailFlow(ctx, rl);
        else if (choice === "9") await initializeSystem(ctx);
      } catch (e) { if (e.message.includes("closed")) break; else console.error("Error:", e.message); }
    }
  } catch (e) { console.error("Fatal:", e.message); } finally { rl.close(); }
}
main();

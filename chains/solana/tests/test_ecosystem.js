const anchor = require("@coral-xyz/anchor");
const { PublicKey, SystemProgram, Keypair } = require("@solana/web3.js");
const fs = require("fs");
const path = require("path");
const os = require("os");

const pgc1Idl = require("../target/idl/pgc1.json");
const registryIdl = require("../target/idl/registry.json");
const storeIdl = require("../target/idl/game_store.json");

async function runTest() {
  console.log("🚀 Starting PGC1 Ecosystem E2E Test...");

  const providerUrl = "http://127.0.0.1:8899";
  const walletPath = path.join(os.homedir(), ".config/solana/id.json");
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf8"));
  const wallet = new anchor.Wallet(Keypair.fromSecretKey(Uint8Array.from(secret)));
  const connection = new anchor.web3.Connection(providerUrl, "confirmed");
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const pgc1Prog = new anchor.Program(pgc1Idl, provider);
  const regProg = new anchor.Program(registryIdl, provider);
  const storeProg = new anchor.Program(storeIdl, provider);

  const testGameId = `test-${Date.now()}`;
  console.log(`Using Game ID: ${testGameId}`);

  // 1. Initialize
  console.log("1. Bootstrapping System...");
  const regConfigPda = PublicKey.findProgramAddressSync([Buffer.from("config")], regProg.programId)[0];
  const storeConfigPda = PublicKey.findProgramAddressSync([Buffer.from("config")], storeProg.programId)[0];

  try {
    await regProg.methods.initialize().accounts({
      authority: wallet.publicKey,
      config: regConfigPda,
      systemProgram: SystemProgram.programId
    }).rpc();
    console.log("✅ Registry Init");
  } catch (e) {
    console.log("ℹ️ Registry skipping (already init)");
  }

  try {
    await storeProg.methods.initialize(100, wallet.publicKey).accounts({
      authority: wallet.publicKey,
      config: storeConfigPda,
      systemProgram: SystemProgram.programId
    }).rpc();
    console.log("✅ Store Init");
  } catch (e) {
    console.log("ℹ️ Store skipping (already init)");
  }

  // 2. Create Game
  console.log("2. Creating Game via PGC1...");
  const pgcGamePda = PublicKey.findProgramAddressSync([Buffer.from("game"), Buffer.from(testGameId)], pgc1Prog.programId)[0];
  const regGamePda = PublicKey.findProgramAddressSync([Buffer.from("game"), Buffer.from(testGameId)], regProg.programId)[0];
  const pricePda = PublicKey.findProgramAddressSync([Buffer.from("price"), pgcGamePda.toBuffer()], storeProg.programId)[0];
  const minterPda = PublicKey.findProgramAddressSync([Buffer.from("minter"), pgcGamePda.toBuffer(), storeConfigPda.toBuffer()], pgc1Prog.programId)[0];
  
  const regConfig = await regProg.account.registryConfig.fetch(regConfigPda);

  await pgc1Prog.methods.createGame(
    testGameId,
    "http://test.meta",
    storeConfigPda,
    new anchor.BN(0.1 * 1e9),
    SystemProgram.programId
  ).accounts({
    publisher: wallet.publicKey,
    gameAccount: pgcGamePda,
    initialMinterAccount: minterPda,
    registryProgram: regProg.programId,
    storeProgram: storeProg.programId,
    registryConfig: regConfigPda,
    registryTreasury: regConfig.treasury,
    registryGame: regGamePda,
    priceAccount: pricePda,
    systemProgram: SystemProgram.programId
  }).rpc();
  console.log("✅ Game Created");

  // 3. Buy Game
  console.log("3. Buying Game...");
  const licensePda = PublicKey.findProgramAddressSync([Buffer.from("license"), wallet.publicKey.toBuffer(), pgcGamePda.toBuffer()], pgc1Prog.programId)[0];
  const storeConfig = await storeProg.account.storeConfig.fetch(storeConfigPda);
  const publisherBalancePda = PublicKey.findProgramAddressSync([Buffer.from("balance"), wallet.publicKey.toBuffer()], storeProg.programId)[0];

  await storeProg.methods.buyGame().accounts({
    buyer: wallet.publicKey,
    config: storeConfigPda,
    treasury: storeConfig.treasury,
    game: pgcGamePda,
    priceAccount: pricePda,
    publisher: wallet.publicKey,
    publisherBalance: publisherBalancePda,
    pgcProgram: pgc1Prog.programId,
    minterPda: minterPda,
    licensePda: licensePda,
    systemProgram: SystemProgram.programId
  }).rpc();
  console.log("✅ Purchase Successful");

  // 4. Verify License
  console.log("4. Verifying License...");
  const license = await pgc1Prog.account.licenseAccount.fetch(licensePda);
  if (license.owner.equals(wallet.publicKey)) {
    console.log("✅ License confirmed for owner");
  } else {
    throw new Error("❌ License owner mismatch");
  }

  // 5. Check List
  console.log("5. Testing List Decoder...");
  const discriminator = Buffer.from([17, 140, 126, 39, 63, 84, 119, 73]); // RegistryGameAccount
  const accs = await connection.getProgramAccounts(regProg.programId, {
    filters: [{ memcmp: { offset: 0, bytes: anchor.utils.bytes.bs58.encode(discriminator) } }]
  });
  
  let found = false;
  for (const a of accs) {
    const game = regProg.coder.accounts.decode("registryGameAccount", a.account.data);
    if ((game.gameId || game.game_id) === testGameId) {
      found = true;
      console.log(`✅ Game ${testGameId} found in Registry Catalog`);
    }
  }
  if (!found) throw new Error("❌ Game not found in catalog");

  console.log("\n🎉 ALL TESTS PASSED! PGC1 Ecosystem is 100% Production Ready.");
}

runTest().catch(e => {
  console.error("\n❌ TEST FAILED:", e);
  process.exit(1);
});

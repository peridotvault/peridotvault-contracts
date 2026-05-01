const anchor = require("@coral-xyz/anchor");
const { Program } = require("@coral-xyz/anchor");
const { Keypair, PublicKey, SystemProgram, Transaction } = require("@solana/web3.js");
const {
  createMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} = require("@solana/spl-token");
const readline = require("readline/promises");
const fs = require("fs");
const os = require("os");
const path = require("path");

const pglIdl = require("../target/idl/pgl1.json");
const registryIdl = require("../target/idl/registry.json");
const storeIdl = require("../target/idl/game_store.json");

const INPUT_EOF = "__EOF__";
const DEFAULT_PLATFORM_FEE_BPS = 1000;
const DEFAULT_REFERRAL_BPS = 200;
const DEFAULT_MAX_REFERRAL_BPS = 5000;

function derivePda(seeds, programId) {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

function u64LeBuffer(value) {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(value);
  return buffer;
}

function pubkeyShort(key) {
  const b58 = key.toBase58();
  return `${b58.slice(0, 6)}...${b58.slice(-6)}`;
}

function lamportsToSol(lamports) {
  return (lamports / anchor.web3.LAMPORTS_PER_SOL).toFixed(4);
}

function statusLabel(status) {
  if (!status || typeof status !== "object") {
    return "unknown";
  }
  return Object.keys(status)[0] || "unknown";
}

function normalizeString(value) {
  return String(value ?? "").trim();
}

async function accountExists(connection, address) {
  return (await connection.getAccountInfo(address)) !== null;
}

async function ask(rl, label, defaultValue = "") {
  const suffix = defaultValue ? ` [default: ${defaultValue}]` : "";
  let raw;
  try {
    raw = await rl.question(`${label}${suffix}: `);
  } catch (error) {
    const code = error && error.code ? String(error.code) : "";
    if (code === "ERR_USE_AFTER_CLOSE") {
      return INPUT_EOF;
    }
    throw error;
  }

  const value = normalizeString(raw);
  if (!value) {
    return defaultValue;
  }
  return value;
}

async function askNumber(rl, label, defaultValue) {
  while (true) {
    const value = await ask(rl, label, String(defaultValue));
    if (value === INPUT_EOF) {
      return null;
    }

    const parsed = Number(value);
    if (Number.isFinite(parsed) && parsed >= 0) {
      return parsed;
    }

    console.log("Input angka tidak valid.");
  }
}

async function askPublicKey(rl, label, defaultValue = "") {
  while (true) {
    const value = await ask(rl, label, defaultValue);
    if (value === INPUT_EOF) {
      return null;
    }

    try {
      return new PublicKey(value);
    } catch {
      console.log("Alamat pubkey tidak valid.");
    }
  }
}

async function pickByNumber(rl, title, items, renderItem) {
  if (items.length === 0) {
    return null;
  }

  console.log(`\n${title}`);
  items.forEach((item, idx) => {
    console.log(`${idx + 1}. ${renderItem(item, idx)}`);
  });

  while (true) {
    const answer = await ask(rl, "Pilih nomor", "1");
    if (answer === INPUT_EOF) {
      return null;
    }

    const n = Number(answer);
    if (Number.isInteger(n) && n >= 1 && n <= items.length) {
      return items[n - 1];
    }
    console.log("Nomor pilihan tidak valid.");
  }
}

async function buildContext() {
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const pglProgram = new Program(pglIdl, provider);
  const registryProgram = new Program(registryIdl, provider);
  const storeProgram = new Program(storeIdl, provider);

  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  return {
    provider,
    pglProgram,
    registryProgram,
    storeProgram,
    pglConfigPda,
    registryConfigPda,
    storeConfigPda,
  };
}

async function ensureRpcReady(ctx) {
  try {
    await ctx.provider.connection.getLatestBlockhash();
  } catch (error) {
    throw new Error(
      `RPC tidak bisa diakses (${process.env.ANCHOR_PROVIDER_URL}). Jalankan 'pnpm anchor:localnet' di terminal lain.`,
    );
  }
}

async function showHeader(ctx) {
  const balance = await ctx.provider.connection.getBalance(ctx.provider.publicKey);

  const networkLabel = process.env.ANCHOR_PROVIDER_URL.includes("devnet") ? "Devnet" : "Localnet";

  console.log("\nPeridotVault Console App");
  console.log(`- Network: ${networkLabel} (${process.env.ANCHOR_PROVIDER_URL})`);
  console.log(`- Your Address: ${ctx.provider.publicKey.toBase58()} | SOL ${lamportsToSol(balance)}`);
  console.log("- PGL1 Program:", ctx.pglProgram.programId.toBase58());
  console.log("- Registry Program:", ctx.registryProgram.programId.toBase58());
  console.log("- Store Program:", ctx.storeProgram.programId.toBase58());
}

async function dashboard(ctx) {
  await showHeader(ctx);

  const pglConfig = await ctx.provider.connection.getAccountInfo(ctx.pglConfigPda);
  const registryConfig = await ctx.provider.connection.getAccountInfo(ctx.registryConfigPda);
  const storeConfig = await ctx.provider.connection.getAccountInfo(ctx.storeConfigPda);

  const games = registryConfig ? await ctx.registryProgram.account.registryGame.all() : [];
  const receipts = await ctx.storeProgram.account.purchaseReceipt.all();
  const licenses = await ctx.pglProgram.account.license.all();

  console.log("\n== Dashboard ==");
  console.log("pgl_config:", pglConfig ? "EXISTS" : "MISSING", ctx.pglConfigPda.toBase58());
  console.log(
    "registry_config:",
    registryConfig ? "EXISTS" : "MISSING",
    ctx.registryConfigPda.toBase58(),
  );
  console.log(
    "store_config:",
    storeConfig ? "EXISTS" : "MISSING",
    ctx.storeConfigPda.toBase58(),
  );
  console.log("registry games:", games.length);
  console.log("purchase receipts:", receipts.length);
  console.log("licenses:", licenses.length);
}

async function getCoreConfigStatus(ctx) {
  const [pgl, registry, store] = await Promise.all([
    accountExists(ctx.provider.connection, ctx.pglConfigPda),
    accountExists(ctx.provider.connection, ctx.registryConfigPda),
    accountExists(ctx.provider.connection, ctx.storeConfigPda),
  ]);

  return { pgl, registry, store };
}

async function bootstrapCoreConfigsFlow(ctx) {
  console.log("\n== Bootstrap: Core Configs ==");
  const status = await getCoreConfigStatus(ctx);

  if (!status.pgl) {
    await ctx.pglProgram.methods
      .initializePgl(ctx.provider.publicKey, new anchor.BN(0))
      .accounts({
        authority: ctx.provider.publicKey,
        pglConfig: ctx.pglConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("pgl_config created.");
  } else {
    console.log("pgl_config already exists.");
  }

  if (!status.registry) {
    await ctx.registryProgram.methods
      .initializeRegistry(ctx.provider.publicKey)
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("registry_config created.");
  } else {
    console.log("registry_config already exists.");
  }

  if (!status.store) {
    await ctx.storeProgram.methods
      .initializeStore(
        ctx.provider.publicKey,
        new anchor.BN(DEFAULT_PLATFORM_FEE_BPS),
        new anchor.BN(DEFAULT_REFERRAL_BPS),
        new anchor.BN(DEFAULT_MAX_REFERRAL_BPS),
        ctx.provider.publicKey,
      )
      .accounts({
        authority: ctx.provider.publicKey,
        storeConfig: ctx.storeConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("store_config created.");
  } else {
    console.log("store_config already exists.");
  }
}

async function getRegistryGameRows(ctx) {
  const entries = await ctx.registryProgram.account.registryGame.all();

  return Promise.all(
    entries.map(async (entry) => {
      let publisher = null;
      try {
        const sourceGame = await ctx.pglProgram.account.game.fetch(entry.account.game);
        publisher = sourceGame.publisher;
      } catch {
        publisher = null;
      }

      return {
        entry,
        status: statusLabel(entry.account.status),
        publisher,
      };
    }),
  );
}

async function listRegistryGamesFlow(ctx) {
  const rows = await getRegistryGameRows(ctx);
  console.log("\n== Registry: All Games ==");

  if (rows.length === 0) {
    console.log("Belum ada game di registry.");
    return;
  }

  rows.forEach((row, idx) => {
    const publisherLabel = row.publisher ? row.publisher.toBase58() : "(unknown)";
    console.log(
      `${idx + 1}. game_id=${row.entry.account.gameId} | status=${row.status} | game=${row.entry.account.game.toBase58()} | publisher=${publisherLabel}`,
    );
  });
}

async function pickRegistryGame(ctx, rl) {
  const rows = await getRegistryGameRows(ctx);
  if (rows.length === 0) {
    console.log("Belum ada game di registry.");
    return null;
  }

  const selected = await pickByNumber(
    rl,
    "Pilih game di registry",
    rows,
    (row) => {
      const publisher = row.publisher ? pubkeyShort(row.publisher) : "unknown";
      return `${row.entry.account.gameId} | status=${row.status} | game=${pubkeyShort(row.entry.account.game)} | pub=${publisher}`;
    },
  );

  return selected;
}

async function listRegistryPaymentTokensFlow(ctx) {
  if (!(await accountExists(ctx.provider.connection, ctx.registryConfigPda))) {
    console.log("\nRegistry config belum ada. Jalankan Main Menu -> Bootstrap Core Configs.");
    return;
  }

  const tokens = await ctx.registryProgram.account.acceptedPaymentToken.all();
  console.log("\n== Registry: Payment Tokens ==");

  if (tokens.length === 0) {
    console.log("Belum ada payment token di registry.");
    return;
  }

  tokens.forEach((token, idx) => {
    console.log(
      `${idx + 1}. mint=${token.account.mint.toBase58()} | active=${token.account.active} | fee=${token.account.feeAmount.toString()}`,
    );
  });
}

async function listStorePaymentTokensFlow(ctx) {
  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    console.log("\nStore config belum ada. Jalankan Main Menu -> Bootstrap Core Configs.");
    return;
  }

  const tokens = await ctx.storeProgram.account.acceptedPaymentToken.all();
  console.log("\n== Store: Payment Tokens ==");

  if (tokens.length === 0) {
    console.log("Belum ada payment token di store.");
    return;
  }

  tokens.forEach((token, idx) => {
    console.log(
      `${idx + 1}. mint=${token.account.mint.toBase58()} | active=${token.account.active}`,
    );
  });
}

async function chooseRegistryAcceptedMint(ctx, rl) {
  if (!(await accountExists(ctx.provider.connection, ctx.registryConfigPda))) {
    throw new Error(
      "registry_config belum ada. Jalankan Main Menu -> Bootstrap Core Configs dulu.",
    );
  }

  const tokens = await ctx.registryProgram.account.acceptedPaymentToken.all();
  const active = tokens.filter((t) => t.account.active);

  if (active.length === 0) {
    throw new Error("Tidak ada accepted payment token aktif di registry.");
  }

  const picked = await pickByNumber(
    rl,
    "Pilih payment mint (registry accepted token)",
    active,
    (item) => {
      const fee = Number(item.account.feeAmount.toString());
      return `${item.account.mint.toBase58()} (fee=${fee})`;
    },
  );

  return picked ? picked.account.mint : null;
}

async function chooseStoreAcceptedMint(ctx, rl) {
  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    throw new Error(
      "store_config belum ada. Jalankan Main Menu -> Bootstrap Core Configs dulu.",
    );
  }

  const tokens = await ctx.storeProgram.account.acceptedPaymentToken.all();
  const active = tokens.filter((t) => t.account.active);

  if (active.length === 0) {
    throw new Error("Tidak ada accepted payment token aktif di store.");
  }

  const picked = await pickByNumber(
    rl,
    "Pilih payment mint (store accepted token)",
    active,
    (item) => item.account.mint.toBase58(),
  );

  return picked ? picked.account.mint : null;
}

async function ensurePublishGrant(ctx, publisherPk) {
  const publishGrantPda = derivePda(
    [Buffer.from("publish_grant"), publisherPk.toBuffer()],
    ctx.registryProgram.programId,
  );

  const grantExists = await accountExists(ctx.provider.connection, publishGrantPda);

  if (!grantExists) {
    await ctx.registryProgram.methods
      .createPublishGrant(null)
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        publisher: publisherPk,
        publishGrant: publishGrantPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  } else {
    await ctx.registryProgram.methods
      .updatePublishGrant(null)
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        publisher: publisherPk,
        publishGrant: publishGrantPda,
      })
      .rpc();
  }

  return publishGrantPda;
}

async function createGameFlow(ctx, rl) {
  console.log("\n== Registry: Create Game + Register ==");

  const status = await getCoreConfigStatus(ctx);
  if (!status.pgl || !status.registry) {
    console.log("pgl_config / registry_config belum siap.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const paymentMint = await chooseRegistryAcceptedMint(ctx, rl);
  if (!paymentMint) {
    return;
  }

  const gameId = await ask(rl, "Game ID", `game-${Date.now()}`);
  if (gameId === INPUT_EOF) {
    return;
  }

  const metadataUri = await ask(rl, "Metadata URI", `https://meta.peridot/${gameId}.json`);
  if (metadataUri === INPUT_EOF) {
    return;
  }

  const isFree = await ask(rl, "Is this game free? (y/n)", "y");
  let basePrice = null;
  let mintToken = null;
  if (isFree.toLowerCase() === "n") {
    const storeMint = await chooseStoreAcceptedMint(ctx, rl);
    if (!storeMint) return;
    const price = await askNumber(rl, "Base price", 20_000_000);
    if (price !== null) {
      basePrice = new anchor.BN(price);
      mintToken = storeMint;
    }
  }

  const creatorStatePda = derivePda(
    [Buffer.from("creator_state"), ctx.provider.publicKey.toBuffer()],
    ctx.pglProgram.programId,
  );

  let nextNonce = 0n;
  const creatorStateExists = await accountExists(ctx.provider.connection, creatorStatePda);
  if (creatorStateExists) {
    const creatorState = await ctx.pglProgram.account.creatorState.fetch(creatorStatePda);
    nextNonce = BigInt(creatorState.nextNonce.toString());
  }

  const gamePda = derivePda(
    [Buffer.from("game"), ctx.provider.publicKey.toBuffer(), u64LeBuffer(nextNonce)],
    ctx.pglProgram.programId,
  );

  const registryGamePda = derivePda(
    [Buffer.from("registry_game"), gamePda.toBuffer()],
    ctx.registryProgram.programId,
  );

  const registryConfig = await ctx.registryProgram.account.registryConfig.fetch(
    ctx.registryConfigPda,
  );
  const pglConfig = await ctx.pglProgram.account.pglConfig.fetch(ctx.pglConfigPda);

  const storeProgramId = ctx.storeProgram.programId;
  const storeAuthorizedSourceProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.pglProgram.programId.toBuffer()],
    storeProgramId,
  );
  const storeAuthorizedRegistryProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.registryProgram.programId.toBuffer()],
    storeProgramId,
  );
  const storeGameStoreConfig = derivePda(
    [Buffer.from("game_store_config"), gamePda.toBuffer()],
    storeProgramId,
  );

  // Always ensure publish grant exists (creates free publisher grant)
  const publishGrantPda = derivePda(
    [Buffer.from("publish_grant"), ctx.provider.publicKey.toBuffer()],
    ctx.registryProgram.programId,
  );

  if (!(await accountExists(ctx.provider.connection, publishGrantPda))) {
    await ctx.registryProgram.methods
      .createPublishGrant(null)
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        publisher: ctx.provider.publicKey,
        publishGrant: publishGrantPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Publish grant created (free publisher).");
  }

  // Load payment info and check balance
  const registryAcceptedPda = derivePda(
    [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
    ctx.registryProgram.programId,
  );
  const registryToken = await ctx.registryProgram.account.acceptedPaymentToken.fetch(registryAcceptedPda);
  console.log("Registration fee:", registryToken.feeAmount.toString());

  const publisherPaymentAta = await getOrCreateAssociatedTokenAccount(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    paymentMint,
    ctx.provider.publicKey,
  );

  const treasuryPaymentAta = await getOrCreateAssociatedTokenAccount(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    paymentMint,
    registryConfig.treasury,
  );

  // Build remaining accounts: [publishGrant] + store accounts
  const remainingAccounts = [{
    pubkey: publishGrantPda,
    isWritable: false,
    isSigner: false,
  }];

  if (basePrice) {
    remainingAccounts.push({
      pubkey: derivePda(
        [Buffer.from("accepted_payment_token"), mintToken.toBuffer()],
        ctx.storeProgram.programId,
      ),
      isWritable: false,
      isSigner: false,
    });
    remainingAccounts.push({
      pubkey: derivePda(
        [Buffer.from("game_payment_option"), gamePda.toBuffer(), mintToken.toBuffer()],
        ctx.storeProgram.programId,
      ),
      isWritable: true,
      isSigner: false,
    });
  }
  console.log("Creating game and registering...");
  console.log("- game PDA:", gamePda.toBase58());
  console.log("- registry_game PDA:", registryGamePda.toBase58());
  console.log("- creator_state PDA:", creatorStatePda.toBase58());
  if (basePrice) {
    console.log("- store mint:", mintToken.toBase58());
    console.log("- base price:", basePrice.toString());
  }
  
  await ctx.registryProgram.methods
      .createGameAndRegister(gameId, metadataUri, basePrice, mintToken)
      .accounts({
        publisher: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        paymentMint,
        acceptedPaymentToken: derivePda(
          [Buffer.from("accepted_payment_token"), paymentMint.toBuffer()],
          ctx.registryProgram.programId,
        ),
        publisherPaymentAccount: publisherPaymentAta.address,
        treasuryPaymentAccount: treasuryPaymentAta.address,
        registryGame: registryGamePda,
        game: gamePda,
        pglCreatorState: creatorStatePda,
        pglConfig: ctx.pglConfigPda,
        pglTreasury: pglConfig.treasury,
        pgl1Program: ctx.pglProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        storeProgram: storeProgramId,
        storeAuthorizedSourceProgram,
        storeAuthorizedRegistryProgram,
        storeGameStoreConfig,
        registryProgram: ctx.registryProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .rpc();

  console.log("Game berhasil dibuat dan diregister.");
  console.log("- game:", gamePda.toBase58());
  console.log("- registry_game:", registryGamePda.toBase58());
  console.log("- status registry default: active (otomatis saat create_game_and_register)");
  if (basePrice) {
    console.log("- store price:", basePrice.toString(), "mint:", mintToken.toBase58());
  }
}

async function promptMintAddressOrCreate(ctx, rl) {
  const mode = await ask(
    rl,
    "Masukkan mint address (kosongkan untuk auto-create mint baru)",
    "",
  );

  if (mode === INPUT_EOF) {
    return null;
  }

  if (mode) {
    try {
      const mint = new PublicKey(mode);
      const info = await ctx.provider.connection.getAccountInfo(mint);
      if (!info) {
        console.log("Mint belum ada on-chain.");
        return null;
      }
      return mint;
    } catch {
      console.log("Mint address tidak valid.");
      return null;
    }
  }

  const decimals = await askNumber(rl, "Decimals mint baru", 6);
  if (decimals === null) {
    return null;
  }

  const mint = await createMint(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    ctx.provider.publicKey,
    null,
    decimals,
  );

  console.log("Mint baru berhasil dibuat:", mint.toBase58());
  return mint;
}

async function addOrUpdateRegistryPaymentTokenFlow(ctx, rl) {
  console.log("\n== Registry: Add / Update Payment Token ==");

  if (!(await accountExists(ctx.provider.connection, ctx.registryConfigPda))) {
    console.log("registry_config belum ada.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const mint = await promptMintAddressOrCreate(ctx, rl);
  if (!mint) {
    return;
  }

  const feeAmount = await askNumber(rl, "Fee amount (u64)", 1_000);
  if (feeAmount === null) {
    return;
  }

  const acceptedPaymentToken = derivePda(
    [Buffer.from("accepted_payment_token"), mint.toBuffer()],
    ctx.registryProgram.programId,
  );

  if (await accountExists(ctx.provider.connection, acceptedPaymentToken)) {
    await ctx.registryProgram.methods
      .updatePaymentToken(true, new anchor.BN(feeAmount))
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        mint,
        acceptedPaymentToken,
      })
      .rpc();
    console.log("Registry payment token berhasil di-update.");
  } else {
    await ctx.registryProgram.methods
      .addPaymentToken(new anchor.BN(feeAmount))
      .accounts({
        authority: ctx.provider.publicKey,
        config: ctx.registryConfigPda,
        mint,
        acceptedPaymentToken,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Registry payment token berhasil ditambahkan.");
  }

  console.log("- mint:", mint.toBase58());
  console.log("- accepted_payment_token:", acceptedPaymentToken.toBase58());
}

async function setRegistryGameStatusFlow(ctx, rl) {
  console.log("\n== Registry: Set Game Status (Admin) ==");

  if (!(await accountExists(ctx.provider.connection, ctx.registryConfigPda))) {
    console.log("registry_config belum ada.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const selected = await pickRegistryGame(ctx, rl);
  if (!selected) {
    return;
  }

  const statuses = [
    { label: "Active", code: 0 },
    { label: "Suspended", code: 1 },
    { label: "Banned", code: 2 },
  ];

  const target = await pickByNumber(
    rl,
    `Pilih status target (current=${selected.status})`,
    statuses,
    (item) => `${item.label} (${item.code})`,
  );

  if (!target) {
    return;
  }

  await ctx.registryProgram.methods
    .updateGameStatus(target.code)
    .accounts({
      authority: ctx.provider.publicKey,
      config: ctx.registryConfigPda,
      registryGame: selected.entry.publicKey,
    })
    .rpc();

  console.log("Status game registry berhasil di-update.");
  console.log("- game_id:", selected.entry.account.gameId);
  console.log("- status baru:", target.label.toLowerCase());
}

async function addOrUpdateStorePaymentTokenFlow(ctx, rl) {
  console.log("\n== Store: Add / Update Payment Token ==");

  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    console.log("store_config belum ada.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const mint = await promptMintAddressOrCreate(ctx, rl);
  if (!mint) {
    return;
  }

  const acceptedPaymentToken = derivePda(
    [Buffer.from("accepted_payment_token"), mint.toBuffer()],
    ctx.storeProgram.programId,
  );

  if (await accountExists(ctx.provider.connection, acceptedPaymentToken)) {
    await ctx.storeProgram.methods
      .updatePaymentToken(true)
      .accounts({
        authority: ctx.provider.publicKey,
        storeConfig: ctx.storeConfigPda,
        acceptedPaymentToken,
      })
      .rpc();
    console.log("Store payment token berhasil diaktifkan/update.");
  } else {
    await ctx.storeProgram.methods
      .addPaymentToken()
      .accounts({
        authority: ctx.provider.publicKey,
        storeConfig: ctx.storeConfigPda,
        mint,
        acceptedPaymentToken,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("Store payment token berhasil ditambahkan.");
  }

  console.log("- mint:", mint.toBase58());
  console.log("- accepted_payment_token:", acceptedPaymentToken.toBase58());
}

async function configureStoreFlow(ctx, rl) {
  console.log("\n== Store: Configure Listing ==");

  const status = await getCoreConfigStatus(ctx);
  if (!status.pgl || !status.registry || !status.store) {
    console.log("Core config belum lengkap.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const selected = await pickRegistryGame(ctx, rl);
  if (!selected) {
    return;
  }

  const gamePk = selected.entry.account.game;
  const game = await ctx.pglProgram.account.game.fetch(gamePk);

  const storeTokens = await ctx.storeProgram.account.acceptedPaymentToken.all();
  const activeStoreTokens = storeTokens.filter((t) => t.account.active);
  if (activeStoreTokens.length === 0) {
    console.log("Tidak ada accepted payment token aktif di store.");
    return;
  }

  const token = await pickByNumber(
    rl,
    "Pilih payment mint (store accepted token)",
    activeStoreTokens,
    (item) => item.account.mint.toBase58(),
  );
  if (!token) {
    return;
  }

  const basePrice = await askNumber(rl, "Base price", 20_000_000);
  if (basePrice === null) {
    return;
  }

  const authorizedSourceProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.pglProgram.programId.toBuffer()],
    ctx.storeProgram.programId,
  );
  const authorizedRegistryProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.registryProgram.programId.toBuffer()],
    ctx.storeProgram.programId,
  );
  const gameStoreConfig = derivePda(
    [Buffer.from("game_store_config"), gamePk.toBuffer()],
    ctx.storeProgram.programId,
  );
  const gamePaymentOption = derivePda(
    [Buffer.from("game_payment_option"), gamePk.toBuffer(), token.account.mint.toBuffer()],
    ctx.storeProgram.programId,
  );

  if (!(await accountExists(ctx.provider.connection, gameStoreConfig))) {
    await ctx.storeProgram.methods
      .initGameStoreConfig(true)
      .accounts({
        payer: ctx.provider.publicKey,
        publisher: ctx.provider.publicKey,
        sourceProgram: ctx.pglProgram.programId,
        authorizedSourceProgram,
        registryProgram: ctx.registryProgram.programId,
        authorizedRegistryProgram,
        game: gamePk,
        registryGame: selected.entry.publicKey,
        gameStoreConfig,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  await ctx.storeProgram.methods
    .setGamePaymentOption(new anchor.BN(basePrice), true)
    .accounts({
      payer: ctx.provider.publicKey,
      publisher: ctx.provider.publicKey,
      sourceProgram: ctx.pglProgram.programId,
      authorizedSourceProgram,
      registryProgram: ctx.registryProgram.programId,
      authorizedRegistryProgram,
      game: gamePk,
      registryGame: selected.entry.publicKey,
      gameStoreConfig,
      mint: token.account.mint,
      acceptedPaymentToken: token.publicKey,
      gamePaymentOption,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("Listing store berhasil di-set.");
  console.log("- game_store_config:", gameStoreConfig.toBase58());
  console.log("- game_payment_option:", gamePaymentOption.toBase58());
}

async function getStoreGamePaymentOptions(ctx) {
  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    return [];
  }
  return ctx.storeProgram.account.gamePaymentOption.all();
}

async function listStorePaymentOptionsFlow(ctx) {
  const options = await getStoreGamePaymentOptions(ctx);
  console.log("\n== Store: Game Payment Options ==");

  if (options.length === 0) {
    console.log("Belum ada game payment option.");
    return;
  }

  options.forEach((item, idx) => {
    let gameId = "(unknown)";
    try {
      // - can't easily get game_id here, just show pubkey
    } catch {}
    console.log(
      `${idx + 1}. game=${item.account.game.toBase58()} | mint=${item.account.mint.toBase58()} | price=${item.account.basePrice.toString()} | active=${item.account.active}`,
    );
  });
}

async function pickGameToBuy(ctx, rl) {
  const storeConfigs = await ctx.storeProgram.account.gameStoreConfig.all();
  const paymentOptions = await getStoreGamePaymentOptions(ctx);
  const paidGames = new Set(paymentOptions.map((o) => o.account.game.toBase58()));

  if (storeConfigs.length === 0) {
    console.log("Belum ada game di store.");
    return null;
  }

  const items = storeConfigs.map((cfg) => {
    const gameAddress = cfg.account.game.toBase58();
    const isPaid = paidGames.has(gameAddress);
    const option = paymentOptions.find((o) => o.account.game.toBase58() === gameAddress);
    return { gamePk: cfg.account.game, isPaid, option };
  });

  const picked = await pickByNumber(
    rl,
    "Pilih game to buy (free/paid)",
    items,
    (item) => {
      const short = pubkeyShort(item.gamePk);
      if (item.isPaid && item.option) {
        return `${short} | PAID | price=${item.option.account.basePrice.toString()} | mint=${pubkeyShort(item.option.account.mint)}`;
      }
      return `${short} | FREE`;
    },
  );

  return picked;
}

async function pickGamePaymentOption(ctx, rl) {
  const options = await getStoreGamePaymentOptions(ctx);
  if (options.length === 0) {
    console.log("Belum ada game payment option di store.");
    return null;
  }

  return pickByNumber(
    rl,
    "Pilih game payment option",
    options,
    (item) =>
      `game=${pubkeyShort(item.account.game)} mint=${pubkeyShort(item.account.mint)} price=${item.account.basePrice.toString()} active=${item.account.active}`,
  );
}

async function buyGameFlow(ctx, rl) {
  console.log("\n== Store: Buy Game ==");

  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    console.log("store_config belum ada.");
    console.log("Jalankan Main Menu -> Bootstrap Core Configs dulu.");
    return;
  }

  const picked = await pickGameToBuy(ctx, rl);
  if (!picked) {
    return;
  }

  const gamePk = picked.gamePk;
  const game = await ctx.pglProgram.account.game.fetch(gamePk);
  const buyer = ctx.provider.publicKey;
  const license = derivePda(
    [Buffer.from("license"), buyer.toBuffer(), gamePk.toBuffer()],
    ctx.pglProgram.programId,
  );

  if (await accountExists(ctx.provider.connection, license)) {
    console.log("You already own a license for this game.");
    return;
  }

  if (!picked.isPaid) {
    // Free game - mint license directly
    console.log("\nFree game — minting license...");
    const authorizedActor = derivePda(
      [Buffer.from("authorized_actor"), buyer.toBuffer()],
      ctx.pglProgram.programId,
    );

    if (!(await accountExists(ctx.provider.connection, authorizedActor))) {
      await ctx.pglProgram.methods
        .addAuthorizedActor()
        .accounts({
          authority: ctx.provider.publicKey,
          actor: buyer,
          pglConfig: ctx.pglConfigPda,
          authorizedActor,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    await ctx.pglProgram.methods
      .mintLicense(null)
      .accounts({
        actor: buyer,
        holder: buyer,
        authorizedActor,
        game: gamePk,
        license,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("License minted!");
    console.log("- license:", license.toBase58());
    return;
  }

  // Paid game
  const option = picked.option;

  const authorizedSourceProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.pglProgram.programId.toBuffer()],
    ctx.storeProgram.programId,
  );
  const authorizedRegistryProgram = derivePda(
    [Buffer.from("authorized_program"), ctx.registryProgram.programId.toBuffer()],
    ctx.storeProgram.programId,
  );
  const gameStoreConfig = derivePda(
    [Buffer.from("game_store_config"), gamePk.toBuffer()],
    ctx.storeProgram.programId,
  );
  const registryGame = derivePda(
    [Buffer.from("registry_game"), gamePk.toBuffer()],
    ctx.registryProgram.programId,
  );
  const acceptedPaymentToken = derivePda(
    [Buffer.from("accepted_payment_token"), option.account.mint.toBuffer()],
    ctx.storeProgram.programId,
  );
  const purchaseReceipt = derivePda(
    [Buffer.from("purchase_receipt"), buyer.toBuffer(), gamePk.toBuffer()],
    ctx.storeProgram.programId,
  );

  const publisherPaymentAccount = await getOrCreateAssociatedTokenAccount(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    option.account.mint,
    game.publisher,
  );

  const storeConfig = await ctx.storeProgram.account.storeConfig.fetch(ctx.storeConfigPda);
  const treasuryPaymentAccount = await getOrCreateAssociatedTokenAccount(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    option.account.mint,
    storeConfig.treasury,
  );

  const buyerPaymentAccount = await getOrCreateAssociatedTokenAccount(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    option.account.mint,
    buyer,
  );

  const price = option.account.basePrice;
  const referrerInput = await ask(rl, "Referrer address (optional, enter to skip)", "");
  let referrer = null;
  let referrerPaymentAccount = null;
  if (referrerInput) {
    try {
      referrer = new PublicKey(referrerInput);
      const rAta = await getOrCreateAssociatedTokenAccount(
        ctx.provider.connection,
        ctx.provider.wallet.payer,
        option.account.mint,
        referrer,
      );
      referrerPaymentAccount = rAta.address;
    } catch {
      console.log("Invalid referrer address, skipping.");
    }
  }

  await ctx.storeProgram.methods
    .buyGame(price, referrer)
    .accounts({
      buyer,
      storeConfig: ctx.storeConfigPda,
      authorizedSourceProgram,
      sourceProgram: ctx.pglProgram.programId,
      authorizedRegistryProgram,
      registryProgram: ctx.registryProgram.programId,
      game: gamePk,
      registryGame,
      gameStoreConfig,
      paymentMint: option.account.mint,
      acceptedPaymentToken,
      gamePaymentOption: option.publicKey,
      buyerPaymentAccount: buyerPaymentAccount.address,
      publisherPaymentAccount: publisherPaymentAccount.address,
      treasuryPaymentAccount: treasuryPaymentAccount.address,
      referrerPaymentAccount,
      storeActor: ctx.provider.publicKey,
      authorizedActor: derivePda(
        [Buffer.from("authorized_actor"), ctx.provider.publicKey.toBuffer()],
        ctx.pglProgram.programId,
      ),
      pgl1Program: ctx.pglProgram.programId,
      license,
      purchaseReceipt,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("- purchase_receipt:", purchaseReceipt.toBase58());
}

async function listPurchaseReceiptsFlow(ctx) {
  if (!(await accountExists(ctx.provider.connection, ctx.storeConfigPda))) {
    console.log("\nStore config belum ada. Jalankan Main Menu -> Bootstrap Core Configs.");
    return;
  }

  const receipts = await ctx.storeProgram.account.purchaseReceipt.all();
  console.log("\n== Store: Purchase Receipts ==");

  if (receipts.length === 0) {
    console.log("Belum ada purchase receipt.");
    return;
  }

  receipts.forEach((item, idx) => {
    console.log(
      `${idx + 1}. buyer=${item.account.buyer.toBase58()} | game=${item.account.game.toBase58()} | paid=${item.account.paidAmount.toString()} | final=${item.account.finalPrice.toString()} | mint=${item.account.paymentMint.toBase58()}`,
    );
  });
}

async function mintLicenseFlow(ctx, rl) {
  console.log("\n== License: Mint ==");

  const selected = await pickRegistryGame(ctx, rl);
  if (!selected) {
    return;
  }

  const gamePk = selected.entry.account.game;
  const holder = ctx.provider.publicKey;
  const actor = ctx.provider.publicKey;

  const authorizedActor = derivePda(
    [Buffer.from("authorized_actor"), actor.toBuffer()],
    ctx.pglProgram.programId,
  );

  const license = derivePda(
    [Buffer.from("license"), holder.toBuffer(), gamePk.toBuffer()],
    ctx.pglProgram.programId,
  );

  await ctx.pglProgram.methods
    .mintLicense(null)
    .accounts({
      actor,
      holder,
      authorizedActor,
      game: gamePk,
      license,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("License berhasil di-mint.");
  console.log("- holder:", holder.toBase58());
  console.log("- license:", license.toBase58());
}

async function myLibraryFlow(ctx) {
  console.log("\n== License: My Library ==");

  const holder = ctx.provider.publicKey;
  const licenses = await ctx.pglProgram.account.license.all([
    {
      memcmp: {
        offset: 8,
        bytes: holder.toBase58(),
      },
    },
  ]);

  if (licenses.length === 0) {
    console.log("Belum ada license untuk gamer.");
    return;
  }

  for (let i = 0; i < licenses.length; i += 1) {
    const lic = licenses[i];
    let gameId = "(unknown)";

    try {
      const game = await ctx.pglProgram.account.game.fetch(lic.account.game);
      gameId = game.gameId;
    } catch {
      gameId = "(unknown)";
    }

    console.log(
      `${i + 1}. gameId=${gameId} | game=${lic.account.game.toBase58()} | license=${lic.publicKey.toBase58()}`,
    );
  }
}

async function closeCreatorStateFlow(ctx) {
  console.log("\n== Cleanup: Close Creator State ==");

  const creatorStatePda = derivePda(
    [Buffer.from("creator_state"), ctx.provider.publicKey.toBuffer()],
    ctx.pglProgram.programId,
  );

  if (!(await accountExists(ctx.provider.connection, creatorStatePda))) {
    console.log("creator_state tidak ada.");
    return;
  }

  const creatorState = await ctx.pglProgram.account.creatorState.fetch(creatorStatePda);
  console.log("creator_state found:");
  console.log("- creator:", creatorState.creator.toBase58());
  console.log("- next_nonce:", creatorState.nextNonce.toString());

  if (creatorState.nextNonce > 0n) {
    console.log("\nWARNING: Creator has existing games. Closing will not remove them.");
    console.log("You may need to close registry_game accounts manually.");
  }

  try {
    await ctx.pglProgram.methods
      .closeCreatorState()
      .accounts({
        creator: ctx.provider.publicKey,
        creatorState: creatorStatePda,
      })
      .rpc();

    console.log("creator_state berhasil ditutup.");
    console.log("SOL refunded to:", ctx.provider.publicKey.toBase58());
  } catch (error) {
    console.log("Gagal menutup creator_state:");
    console.log(String(error));
  }
}

async function showMainMenu() {
  console.log("\n=== Main Menu ===");
  console.log("1. Dashboard");
  console.log("2. Registry Menu");
  console.log("3. Store Menu");
  console.log("4. License Menu");
  console.log("5. Bootstrap Core Configs");
  console.log("0. Exit");
}

async function showRegistryMenu() {
  console.log("\n=== Registry Menu ===");
  console.log("1. Get All Games");
  console.log("2. Create Game + Register");
  console.log("3. Add / Update Payment Token");
  console.log("4. Set Game Status (Admin)");
  console.log("5. List Payment Tokens");
  console.log("0. Back");
}

async function showStoreMenu() {
  console.log("\n=== Store Menu ===");
  console.log("1. Add / Update Payment Token");
  console.log("2. Configure Game Listing");
  console.log("3. List Game Payment Options");
  console.log("4. Buy Game");
  console.log("5. List Purchase Receipts");
  console.log("6. List Payment Tokens");
  console.log("0. Back");
}

async function showLicenseMenu() {
  console.log("\n=== License Menu ===");
  console.log("1. Mint License");
  console.log("2. My Library");
  console.log("3. Close Creator State (Cleanup)");
  console.log("0. Back");
}

async function registryMenu(ctx, rl) {
  while (true) {
    await showRegistryMenu();
    const choice = await ask(rl, "Pilih registry menu", "1");

    if (choice === INPUT_EOF || choice === "0") {
      return;
    }

    try {
      if (choice === "1") {
        await listRegistryGamesFlow(ctx);
      } else if (choice === "2") {
        await createGameFlow(ctx, rl);
      } else if (choice === "3") {
        await addOrUpdateRegistryPaymentTokenFlow(ctx, rl);
      } else if (choice === "4") {
        await setRegistryGameStatusFlow(ctx, rl);
      } else if (choice === "5") {
        await listRegistryPaymentTokensFlow(ctx);
      } else {
        console.log("Pilihan menu tidak valid.");
      }
    } catch (error) {
      console.log("Transaksi registry gagal:");
      console.log(String(error));
    }
  }
}

async function storeMenu(ctx, rl) {
  while (true) {
    await showStoreMenu();
    const choice = await ask(rl, "Pilih store menu", "1");

    if (choice === INPUT_EOF || choice === "0") {
      return;
    }

    try {
      if (choice === "1") {
        await addOrUpdateStorePaymentTokenFlow(ctx, rl);
      } else if (choice === "2") {
        await configureStoreFlow(ctx, rl);
      } else if (choice === "3") {
        await listStorePaymentOptionsFlow(ctx);
      } else if (choice === "4") {
        await buyGameFlow(ctx, rl);
      } else if (choice === "5") {
        await listPurchaseReceiptsFlow(ctx);
      } else if (choice === "6") {
        await listStorePaymentTokensFlow(ctx);
      } else {
        console.log("Pilihan menu tidak valid.");
      }
    } catch (error) {
      console.log("Transaksi store gagal:");
      console.log(String(error));
    }
  }
}

async function licenseMenu(ctx, rl) {
  while (true) {
    await showLicenseMenu();
    const choice = await ask(rl, "Pilih license menu", "1");

    if (choice === INPUT_EOF || choice === "0") {
      return;
    }

    try {
      if (choice === "1") {
        await mintLicenseFlow(ctx, rl);
      } else if (choice === "2") {
        await myLibraryFlow(ctx);
      } else if (choice === "3") {
        await closeCreatorStateFlow(ctx);
      } else {
        console.log("Pilihan menu tidak valid.");
      }
    } catch (error) {
      console.log("Transaksi license gagal:");
      console.log(String(error));
    }
  }
}

async function showNetworkSelection() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    console.log("\n=== Select Network ===");
    console.log("1. Localnet (http://127.0.0.1:8899)");
    console.log("2. Devnet (https://api.devnet.solana.com)");

    const choice = await rl.question("Pilih network [default: 1]: ");
    const normalized = String(choice ?? "").trim();

    if (normalized === "2") {
      process.env.ANCHOR_PROVIDER_URL = "https://api.devnet.solana.com";
      console.log("\nUsing Devnet.");
    } else {
      process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";
      console.log("\nUsing Localnet.");
    }

    return normalized === "2" ? "devnet" : "localnet";
  } finally {
    rl.close();
  }
}

async function main() {
  const network = await showNetworkSelection();

  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    const ctx = await buildContext();
    await ensureRpcReady(ctx);
    await dashboard(ctx);

    while (true) {
      await showMainMenu();
      const choice = await ask(rl, "Pilih main menu", "1");

      if (choice === INPUT_EOF || choice === "0") {
        break;
      }

      try {
        if (choice === "1") {
          await dashboard(ctx);
        } else if (choice === "2") {
          await registryMenu(ctx, rl);
        } else if (choice === "3") {
          await storeMenu(ctx, rl);
        } else if (choice === "4") {
          await licenseMenu(ctx, rl);
        } else if (choice === "5") {
          await bootstrapCoreConfigsFlow(ctx);
        } else {
          console.log("Pilihan menu tidak valid.");
        }
      } catch (error) {
        console.log("Operasi gagal:");
        console.log(String(error));
      }
    }
  } finally {
    rl.close();
  }
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

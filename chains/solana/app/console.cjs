const anchor = require("@coral-xyz/anchor");
const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} = require("@solana/spl-token");
const { Keypair, PublicKey, SystemProgram, Transaction } = require("@solana/web3.js");
const { createHash } = require("crypto");
const fs = require("fs");
const os = require("os");
const path = require("path");
const readline = require("readline/promises");
const { stdin: input, stdout: output } = require("process");

const factoryIdl = require("../target/idl/factory.json");
const pgcIdl = require("../target/idl/pgc1.json");
const registryIdl = require("../target/idl/registry.json");
const storeIdl = require("../target/idl/game_store.json");

const REGISTRY_STATE_SEED = Buffer.from("registry_state");
const GAME_REGISTRATION_SEED = Buffer.from("game_registration");
const STORE_STATE_SEED = Buffer.from("game_store_state");
const FACTORY_STATE_SEED = Buffer.from("factory_state");
const FACTORY_MINT_SEED = Buffer.from("factory_mint");
const GAME_STATE_SEED = Buffer.from("game_state");
const GAME_AUTHORITY_SEED = Buffer.from("game_authority");
const MINTER_AUTH_SEED = Buffer.from("minter_auth");
const LICENSE_SEED = Buffer.from("license");

const STATUS_PENDING = 0
const STATUS_APPROVED = 1;
const STATUS_BANNED = 2;

const PAYMENT_DECIMALS = 6;
const DEFAULT_PLATFORM_FEE_BPS = 1000;
const DEFAULT_REGISTRATION_FEE = 5_000_000;
const DEFAULT_METADATA_URI_BASE = "https://peridot.local/metadata";
const FAUCET_AMOUNT = 20_000_000;

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

function deterministicKeypair(label) {
  const seed = createHash("sha256").update(label).digest().subarray(0, 32);
  return Keypair.fromSeed(seed);
}

function derivePda(seeds, programId) {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

function sha256Seed(value) {
  return createHash("sha256").update(value).digest();
}

function formatStatus(status) {
  if (status === STATUS_PENDING) return "Pending";
  if (status === STATUS_APPROVED) return "Approved";
  if (status === STATUS_BANNED) return "Banned";
  return `Unknown(${status})`;
}

function formatAmount(amount) {
  return (Number(amount) / 10 ** PAYMENT_DECIMALS).toFixed(2);
}

function formatSolAmount(lamports) {
  return (Number(lamports) / anchor.web3.LAMPORTS_PER_SOL).toFixed(4);
}

function isNativeSolPaymentMethod(paymentMethod) {
  return paymentMethod.equals(SystemProgram.programId);
}

function formatPaymentMethod(paymentMethod) {
  return isNativeSolPaymentMethod(paymentMethod) ? "SOL" : shortAddress(paymentMethod);
}

function formatPricedAmount(amount, currency) {
  if (amount === null) {
    return "-";
  }
  return isNativeSolPaymentMethod(currency)
    ? `${formatSolAmount(amount)} SOL`
    : `${formatAmount(amount)} tokens`;
}

async function sleep(ms) {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

async function promptNumber(rl, message, fallback) {
  const answer = (await rl.question(message)).trim();
  if (!answer && fallback !== undefined) {
    return fallback;
  }
  const parsed = Number(answer);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Invalid number: ${answer}`);
  }
  return parsed;
}

async function accountExists(connection, address) {
  return (await connection.getAccountInfo(address)) !== null;
}

async function sendLamports(provider, to, lamports) {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: provider.publicKey,
      toPubkey: to,
      lamports,
    }),
  );
  await provider.sendAndConfirm(tx);
}

async function maybeFundSigner(provider, signer) {
  const balance = await provider.connection.getBalance(signer.publicKey);
  if (balance === 0) {
    await sendLamports(provider, signer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
  }
}

async function fetchTokenAmount(connection, address, programId = TOKEN_PROGRAM_ID) {
  const account = await getAccount(connection, address, undefined, programId);
  return Number(account.amount);
}

function makePrograms(provider) {
  const factoryProgram = new anchor.Program(factoryIdl, provider);
  const pgcProgram = new anchor.Program(pgcIdl, provider);
  const registryProgram = new anchor.Program(registryIdl, provider);
  const storeProgram = new anchor.Program(storeIdl, provider);
  return {
    factoryProgram,
    pgcProgram,
    registryProgram,
    storeProgram,
  };
}

async function createPaymentMintAndAtas(provider, treasuryPubkey, userPubkey) {
  const payer = provider.wallet.payer;
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
      treasuryPubkey,
    )
  ).address;
  const userPaymentTokenAccount = (
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      payer,
      paymentMint,
      userPubkey,
    )
  ).address;
  return {
    paymentMint,
    treasuryPaymentTokenAccount,
    userPaymentTokenAccount,
  };
}

function deriveGameAccounts(ctx, gameId) {
  const mintPda = derivePda(
    [FACTORY_MINT_SEED, sha256Seed(gameId)],
    ctx.factoryProgram.programId,
  );
  const gameStatePda = derivePda(
    [GAME_STATE_SEED, Buffer.from(gameId)],
    ctx.pgcProgram.programId,
  );
  const gameAuthorityPda = derivePda(
    [GAME_AUTHORITY_SEED, gameStatePda.toBuffer()],
    ctx.pgcProgram.programId,
  );
  const publisherMinterAuthPda = derivePda(
    [MINTER_AUTH_SEED, gameStatePda.toBuffer(), ctx.user.publicKey.toBuffer()],
    ctx.pgcProgram.programId,
  );
  const storeMinterAuthPda = derivePda(
    [MINTER_AUTH_SEED, gameStatePda.toBuffer(), ctx.storeStatePda.toBuffer()],
    ctx.pgcProgram.programId,
  );
  const licensePda = derivePda(
    [LICENSE_SEED, gameStatePda.toBuffer(), ctx.user.publicKey.toBuffer()],
    ctx.pgcProgram.programId,
  );
  const userGameTokenAccount = getAssociatedTokenAddressSync(
    mintPda,
    ctx.user.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
  );
  const storeVaultTokenAccount = getAssociatedTokenAddressSync(
    ctx.paymentMint,
    ctx.storeStatePda,
    true,
    TOKEN_PROGRAM_ID,
  );
  const gameRegistrationPda = derivePda(
    [GAME_REGISTRATION_SEED, Buffer.from(gameId)],
    ctx.registryProgram.programId,
  );

  return {
    mintPda,
    gameStatePda,
    gameAuthorityPda,
    publisherMinterAuthPda,
    storeMinterAuthPda,
    licensePda,
    userGameTokenAccount,
    storeVaultTokenAccount,
    gameRegistrationPda,
  };
}

async function getCatalog(ctx) {
  const registrations = await ctx.registryProgram.account.gameRegistration.all();
  const storeState = await ctx.storeProgram.account.storeState.fetch(ctx.storeStatePda);

  return registrations.map((reg) => {
    const game = reg.account;
    const priceConfig = storeState.prices.find((entry) => entry.gameId === game.gameId);
    const price = priceConfig ? Number(priceConfig.price.toString()) : null;
    const currency = priceConfig ? priceConfig.currency : null;
    const discountBps = priceConfig ? priceConfig.discountBps : null;
    const finalPrice =
      price === null || discountBps === null
        ? null
        : price - Math.floor((price * discountBps) / 10_000);
    return {
      gameId: game.gameId,
      contractAddress: game.contractAddress,
      status: game.status,
      price,
      currency,
      discountBps,
      finalPrice,
    };
  });
}

async function withdrawFlow(ctx, rl) {
  const storeState = await ctx.storeProgram.account.storeState.fetch(ctx.storeStatePda);
  const balances = storeState.publisherBalances.filter(
    (b) => b.publisher.equals(ctx.user.publicKey) && b.amount.toNumber() > 0,
  );

  if (balances.length === 0) {
    console.log("you have no balance to withdraw");
    return;
  }

  console.log("");
  console.log("your balances");
  balances.forEach((b, index) => {
    console.log(
      `${index + 1}. ${formatPaymentMethod(b.token)}: ${
        isNativeSolPaymentMethod(b.token)
          ? `${formatSolAmount(b.amount)} SOL`
          : `${formatAmount(b.amount)} tokens`
      }`,
    );
  });
  console.log("");

  const choiceInput = (await rl.question(`choose balance to withdraw (1-${balances.length}): `)).trim();
  const balance = balances[Number(choiceInput) - 1];
  if (!balance) {
    console.log("invalid choice");
    return;
  }

  const token = balance.token;
  const isSol = isNativeSolPaymentMethod(token);

  let withdrawAccounts = {
    publisher: ctx.user.publicKey,
    storeState: ctx.storeStatePda,
    systemProgram: SystemProgram.programId,
  };

  if (!isSol) {
    const accounts = deriveGameAccounts(ctx, "any"); // just for vault derivation
    const userAta = getAssociatedTokenAddressSync(token, ctx.user.publicKey);
    const storeVault = getAssociatedTokenAddressSync(token, ctx.storeStatePda, true);

    withdrawAccounts = {
      ...withdrawAccounts,
      paymentMint: token,
      publisherTokenAccount: userAta,
      storeVaultTokenAccount: storeVault,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    };
  }

  const tx = await ctx.storeProgram.methods
    .withdraw(token)
    .accounts(withdrawAccounts)
    .signers([ctx.user])
    .rpc();

  console.log(`withdrawn ${isSol ? formatSolAmount(balance.amount) : formatAmount(balance.amount)} ${isSol ? "SOL" : "tokens"}`);
  console.log(`tx: ${tx}`);
}

async function getMyGames(ctx) {
  const catalog = await getCatalog(ctx);
  const now = Math.floor(Date.now() / 1000);
  const owned = [];

  for (const game of catalog) {
    const licenseAddress = derivePda(
      [LICENSE_SEED, game.contractAddress.toBuffer(), ctx.user.publicKey.toBuffer()],
      ctx.pgcProgram.programId,
    );
    if (!(await accountExists(ctx.provider.connection, licenseAddress))) {
      continue;
    }
    const license = await ctx.pgcProgram.account.licenseAccount.fetch(licenseAddress);
    const expiresAt = Number(license.expiresAt.toString());
    if (expiresAt === 0 || expiresAt > now) {
      owned.push({
        ...game,
        licenseAddress,
      });
    }
  }

  return owned;
}

async function bootstrap() {
  const provider = loadProvider();
  anchor.setProvider(provider);

  try {
    await provider.connection.getVersion();
  } catch (error) {
    throw new Error(
      "Local validator is not reachable. Start it with `pnpm run anchor:localnet` in chains/solana.",
    );
  }

  const programs = makePrograms(provider);
  const governance = provider.wallet.payer;
  const treasury = deterministicKeypair("peridot-localnet-console-treasury");
  const user = deterministicKeypair("peridot-localnet-console-user");

  await maybeFundSigner(provider, user);

  const registryStatePda = derivePda([REGISTRY_STATE_SEED], programs.registryProgram.programId);
  const storeStatePda = derivePda([STORE_STATE_SEED], programs.storeProgram.programId);
  const factoryStatePda = derivePda([FACTORY_STATE_SEED], programs.factoryProgram.programId);

  let paymentMint;
  let treasuryAddress = treasury.publicKey;
  let treasuryPaymentTokenAccount;
  let userPaymentTokenAccount;

  if (await accountExists(provider.connection, registryStatePda)) {
    const registryState = await programs.registryProgram.account.registryState.fetch(registryStatePda);
    treasuryAddress = registryState.treasury;
    const tokenFeeOption = registryState.registrationFeeOptions.find(
      (entry) => !isNativeSolPaymentMethod(entry.paymentMethod),
    );
    if (tokenFeeOption) {
      paymentMint = tokenFeeOption.paymentMethod;
    } else {
      const created = await createPaymentMintAndAtas(provider, treasuryAddress, user.publicKey);
      paymentMint = created.paymentMint;
      treasuryPaymentTokenAccount = created.treasuryPaymentTokenAccount;
      userPaymentTokenAccount = created.userPaymentTokenAccount;
    }
  } else {
    const created = await createPaymentMintAndAtas(provider, treasuryAddress, user.publicKey);
    paymentMint = created.paymentMint;
    treasuryPaymentTokenAccount = created.treasuryPaymentTokenAccount;
    userPaymentTokenAccount = created.userPaymentTokenAccount;
  }

  if (!userPaymentTokenAccount) {
    userPaymentTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        paymentMint,
        user.publicKey,
      )
    ).address;
  }

  if (!treasuryPaymentTokenAccount) {
    treasuryPaymentTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        paymentMint,
        treasuryAddress,
      )
    ).address;
  }

  if (!(await accountExists(provider.connection, registryStatePda))) {
    await programs.registryProgram.methods
      .initialize(
        governance.publicKey,
        treasuryAddress,
        factoryStatePda,
        new anchor.BN(DEFAULT_REGISTRATION_FEE),
        paymentMint,
      )
      .accounts({
        payer: provider.publicKey,
        registryState: registryStatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, storeStatePda))) {
    await programs.storeProgram.methods
      .initialize(
        governance.publicKey,
        treasuryAddress,
        registryStatePda,
        DEFAULT_PLATFORM_FEE_BPS,
      )
      .accounts({
        payer: provider.publicKey,
        storeState: storeStatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  if (!(await accountExists(provider.connection, factoryStatePda))) {
    await programs.factoryProgram.methods
      .initialize(governance.publicKey, registryStatePda, storeStatePda)
      .accounts({
        payer: provider.publicKey,
        factoryState: factoryStatePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  const ctx = {
    provider,
    governance,
    treasury: treasuryAddress,
    user,
    paymentMint,
    treasuryPaymentTokenAccount,
    userPaymentTokenAccount,
    registryStatePda,
    storeStatePda,
    factoryStatePda,
    ...programs,
  };

  return ctx;
}

async function createGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id: ")).trim();
  if (!gameId) {
    console.log("game id is required");
    return;
  }

  const metadataInput = (await rl.question("enter metadata uri (blank = auto): ")).trim();
  const metadataUri = metadataInput || `${DEFAULT_METADATA_URI_BASE}/${gameId}.json`;
  const price = await promptNumber(rl, "enter price in token base units (e.g. 20000000): ", 0);
  const discountBps = await promptNumber(rl, "enter discount bps (0-10000): ", 0);
  const userBalance = await fetchTokenAmount(
    ctx.provider.connection,
    ctx.userPaymentTokenAccount,
    TOKEN_PROGRAM_ID,
  );
  const registryState = await ctx.registryProgram.account.registryState.fetch(ctx.registryStatePda);
  const isFeeExempt = registryState.feeExemptions.some((entry) =>
    entry.equals(ctx.user.publicKey),
  );

  const existingCatalog = await getCatalog(ctx);
  if (existingCatalog.some((game) => game.gameId === gameId)) {
    console.log(`game ${gameId} already exists`);
    return;
  }

  let registrationPaymentMethod = ctx.paymentMint;
  if (!isFeeExempt && registryState.registrationFeeOptions.length > 0) {
    console.log("");
    console.log("registration fee options");
    registryState.registrationFeeOptions.forEach((entry, index) => {
      console.log(
        `${index + 1}. ${formatPaymentMethod(entry.paymentMethod)} - ${
          isNativeSolPaymentMethod(entry.paymentMethod)
            ? `${formatSolAmount(entry.amount)} SOL`
            : `${formatAmount(entry.amount)} tokens`
        }`,
      );
    });
    console.log("");

    const optionInput = (
      await rl.question(`choose payment method (1-${registryState.registrationFeeOptions.length}): `)
    ).trim();
    const optionIndex = Number(optionInput) - 1;
    const feeOption = registryState.registrationFeeOptions[optionIndex];
    if (!feeOption) {
      console.log("invalid payment method");
      return;
    }

    registrationPaymentMethod = feeOption.paymentMethod;
    const requiredAmount = Number(feeOption.amount.toString());
    if (isNativeSolPaymentMethod(registrationPaymentMethod)) {
      const solBalance = await ctx.provider.connection.getBalance(ctx.user.publicKey);
      if (solBalance < requiredAmount) {
        console.log("insufficient SOL balance for registration fee");
        console.log(`required: ${formatSolAmount(requiredAmount)} SOL`);
        console.log(`current : ${formatSolAmount(solBalance)} SOL`);
        return;
      }
    } else if (userBalance < requiredAmount) {
      console.log("insufficient payment token balance for registration fee");
      console.log(`required: ${formatAmount(requiredAmount)} tokens`);
      console.log(`current : ${formatAmount(userBalance)} tokens`);
      console.log("ask governance to reduce the registration fee or grant a fee exemption");
      return;
    }
  }

  const accounts = deriveGameAccounts(ctx, gameId);

  const isSolRegistration = isNativeSolPaymentMethod(registrationPaymentMethod);
  const createAccounts = {
    publisher: ctx.user.publicKey,
    factoryState: ctx.factoryStatePda,
    mint: accounts.mintPda,
    pgcProgram: ctx.pgcProgram.programId,
    pgcGameState: accounts.gameStatePda,
    pgcGameAuthority: accounts.gameAuthorityPda,
    publisherMinterAuth: accounts.publisherMinterAuthPda,
    gameStoreMinterAuth: accounts.storeMinterAuthPda,
    registryProgram: ctx.registryProgram.programId,
    registryState: ctx.registryStatePda,
    gameStoreProgram: ctx.storeProgram.programId,
    treasury: ctx.treasury,
    gameStore: ctx.storeStatePda,
    licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    gameRegistration: accounts.gameRegistrationPda,
    ...(isSolRegistration
      ? {}
      : {
          publisherFeeTokenAccount: ctx.userPaymentTokenAccount,
          treasuryFeeTokenAccount: ctx.treasuryPaymentTokenAccount,
          feePaymentMint: ctx.paymentMint,
          paymentTokenProgram: TOKEN_PROGRAM_ID,
          priceCurrencyMint: ctx.paymentMint,
        }),
  };

  const tx = await ctx.factoryProgram.methods
    .createGame(
      gameId,
      metadataUri,
      new anchor.BN(price),
      ctx.paymentMint,
      registrationPaymentMethod,
    )
    .accounts(createAccounts)
    .signers([ctx.user])
    .rpc();

  await ctx.registryProgram.methods
    .setStatus(gameId, STATUS_APPROVED)
    .accounts({
      admin: ctx.governance.publicKey,
      registryState: ctx.registryStatePda,
      gameRegistration: accounts.gameRegistrationPda,
    })
    .rpc();

  if (discountBps > 0) {
    await ctx.storeProgram.methods
      .setDiscount(gameId, discountBps)
      .accounts({
        publisher: ctx.user.publicKey,
        storeState: ctx.storeStatePda,
        registryState: ctx.registryStatePda,
        pgcGameState: accounts.gameStatePda,
        gameRegistration: accounts.gameRegistrationPda,
      })
      .signers([ctx.user])
      .rpc();
  }

  console.log(`created game ${gameId}`);
  console.log(`tx: ${tx}`);
  console.log(`game state: ${accounts.gameStatePda.toBase58()}`);
  console.log(`status: Approved`);
  console.log(`price : ${formatAmount(price)} tokens`);
}

async function buyGameFlow(ctx, rl) {
  const gameId = (await rl.question("enter game id: ")).trim();
  if (!gameId) {
    console.log("game id is required");
    return;
  }

  const catalog = await getCatalog(ctx);
  const game = catalog.find((entry) => entry.gameId === gameId);
  if (!game) {
    console.log(`game ${gameId} not found`);
    return;
  }
  if (game.status !== STATUS_APPROVED) {
    console.log(`game ${gameId} is ${formatStatus(game.status)} and cannot be bought`);
    return;
  }
  if (game.finalPrice === null) {
    console.log(`game ${gameId} has no price configured`);
    return;
  }
  const isSolPriced = game.currency && isNativeSolPaymentMethod(game.currency);
  const userBalance = isSolPriced
    ? await ctx.provider.connection.getBalance(ctx.user.publicKey)
    : await fetchTokenAmount(
        ctx.provider.connection,
        ctx.userPaymentTokenAccount,
        TOKEN_PROGRAM_ID,
      );
  if (userBalance < game.finalPrice) {
    console.log(isSolPriced ? "insufficient SOL balance" : "insufficient payment token balance");
    console.log(`required: ${formatPricedAmount(game.finalPrice, game.currency)}`);
    console.log(`current : ${formatPricedAmount(userBalance, game.currency)}`);
    return;
  }

  const accounts = deriveGameAccounts(ctx, gameId);

  if (await accountExists(ctx.provider.connection, accounts.licensePda)) {
    const license = await ctx.pgcProgram.account.licenseAccount.fetch(accounts.licensePda);
    const expiresAt = Number(license.expiresAt.toString());
    const now = Math.floor(Date.now() / 1000);
    if (expiresAt === 0 || expiresAt > now) {
      console.log(`you already own ${gameId}`);
      return;
    }
  }

  const buyAccounts = {
    buyer: ctx.user.publicKey,
    storeState: ctx.storeStatePda,
    registryState: ctx.registryStatePda,
    pgcProgram: ctx.pgcProgram.programId,
    pgcGameState: accounts.gameStatePda,
    gameAuthority: accounts.gameAuthorityPda,
    storeMinterAuth: accounts.storeMinterAuthPda,
    licenseAccount: accounts.licensePda,
    userGameTokenAccount: accounts.userGameTokenAccount,
    gameMint: accounts.mintPda,
    treasury: ctx.treasury,
    licenseTokenProgram: TOKEN_2022_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    gameRegistration: accounts.gameRegistrationPda,
    ...(isSolPriced
      ? {}
      : {
          paymentMint: game.currency,
          buyerPaymentTokenAccount: ctx.userPaymentTokenAccount,
          treasuryTokenAccount: ctx.treasuryPaymentTokenAccount,
          storeVaultTokenAccount: accounts.storeVaultTokenAccount,
          paymentTokenProgram: TOKEN_PROGRAM_ID,
        }),
  };

  const tx = await ctx.storeProgram.methods
    .buyGame(gameId)
    .accounts(buyAccounts)
    .signers([ctx.user])
    .rpc();

  console.log(`bought ${gameId}`);
  console.log(`paid: ${formatPricedAmount(game.finalPrice, game.currency)}`);
  console.log(`tx: ${tx}`);
  await sleep(500);
}

async function faucetFlow(ctx) {
  await mintTo(
    ctx.provider.connection,
    ctx.provider.wallet.payer,
    ctx.paymentMint,
    ctx.userPaymentTokenAccount,
    ctx.provider.wallet.payer,
    FAUCET_AMOUNT,
  );

  console.log(`minted ${formatAmount(FAUCET_AMOUNT)} tokens to ${shortAddress(ctx.user.publicKey)}`);
}

async function printAllGames(ctx) {
  const catalog = await getCatalog(ctx);
  if (catalog.length === 0) {
    console.log("no games registered");
    return;
  }

  console.log("");
  console.log("all games");
  for (const game of catalog) {
    console.log(`- gameId: ${game.gameId}`);
    console.log(`  status: ${formatStatus(game.status)}`);
    console.log(`  contract: ${game.contractAddress.toBase58()}`);
    console.log(`  currency: ${game.currency === null ? "-" : formatPaymentMethod(game.currency)}`);
    console.log(`  price: ${formatPricedAmount(game.price, game.currency)}`);
    console.log(`  discount: ${game.discountBps === null ? "-" : `${game.discountBps} bps`}`);
    console.log(`  final price: ${formatPricedAmount(game.finalPrice, game.currency)}`);
  }
}

async function printMyGames(ctx) {
  const games = await getMyGames(ctx);
  if (games.length === 0) {
    console.log("you do not own any games yet");
    return;
  }

  console.log("");
  console.log("my games");
  for (const game of games) {
    console.log(`- gameId: ${game.gameId}`);
    console.log(`  contract: ${game.contractAddress.toBase58()}`);
    console.log(`  status: ${formatStatus(game.status)}`);
    console.log(`  currency: ${game.currency === null ? "-" : formatPaymentMethod(game.currency)}`);
    console.log(`  final price: ${formatPricedAmount(game.finalPrice, game.currency)}`);
    console.log(`  license: ${game.licenseAddress.toBase58()}`);
  }
}

async function printHeader(ctx) {
  const userTokenBalance = await fetchTokenAmount(
    ctx.provider.connection,
    ctx.userPaymentTokenAccount,
    TOKEN_PROGRAM_ID,
  );
  console.log("");
  console.log("Peridot Local Console");
  console.log(`address : ${ctx.user.publicKey.toBase58()}`);
  console.log(`short   : ${shortAddress(ctx.user.publicKey)}`);
  console.log(`balance : ${formatAmount(userTokenBalance)} tokens`);
  console.log("");
  console.log("1. look all games");
  console.log("2. my games");
  console.log("3. buy game");
  console.log("4. create game");
  console.log("5. withdraw");
  console.log("6. Faucet 20 Sol");
  console.log("0. exit");
  console.log("");
}

async function main() {
  const rl = readline.createInterface({ input, output });
  try {
    const ctx = await bootstrap();
    console.log("localnet connected");
    console.log(`registry: ${ctx.registryStatePda.toBase58()}`);
    console.log(`store   : ${ctx.storeStatePda.toBase58()}`);
    console.log(`factory : ${ctx.factoryStatePda.toBase58()}`);
    console.log(`mint    : ${ctx.paymentMint.toBase58()}`);

    while (true) {
      await printHeader(ctx);
      let choice;
      try {
        choice = (await rl.question("choose menu: ")).trim().toLowerCase();
      } catch (error) {
        if ((error.message || "").includes("readline was closed")) {
          break;
        }
        throw error;
      }

      try {
        if (choice === "0" || choice === "q" || choice === "exit") {
          break;
        } else if (choice === "1") {
          await printAllGames(ctx);
        } else if (choice === "2") {
          await printMyGames(ctx);
        } else if (choice === "3") {
          await buyGameFlow(ctx, rl);
        } else if (choice === "4") {
          await createGameFlow(ctx, rl);
        } else if (choice === "5") {
          await withdrawFlow(ctx, rl);
        } else if (choice === "6") {
          await faucetFlow(ctx);
        } else {
          console.log("unknown menu");
        }
      } catch (error) {
        console.error("action failed:");
        console.error(error.message ?? error);
      }
    }
  } finally {
    rl.close();
  }
}

main().catch((error) => {
  console.error(error.message ?? error);
  process.exitCode = 1;
});

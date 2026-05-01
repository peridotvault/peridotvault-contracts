import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

// ============================================================
// HOW TO RUN 
// ============================================================
// ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/config.ts
// ANCHOR_WALLET=~/.config/solana/id.json \
// npx ts-node scripts/config.ts

// ============================================================
//  Peridot Vault — Solana Program Configuration Script
// ============================================================
//  This script initializes and links the three core programs:
//    1. pgl1       — FoLXTWN4iJ9XrmJgfKJBBNmDeYZQt6CL56ftbMkPH4Ky
//    2. registry   — 2HvbxbkJemgFEbdwTLHdQcXb2tRNWxMeCVZ42Gv1kmEA
//    3. game-store — 5uvHYBATc5NURhckg5uL2BQiVzaSMjJQegkhdLMDRe7E
//
//  Run:
//    npx ts-node scripts/config.ts
//
//  Environment overrides:
//    ANCHOR_PROVIDER_URL         default: http://127.0.0.1:8899
//    ANCHOR_WALLET               default: ~/.config/solana/id.json
//    PERIDOT_TREASURY            default: <wallet pubkey>
//    PERIDOT_STORE_ACTOR         default: <wallet pubkey>
//    PERIDOT_PLATFORM_FEE_BPS    default: 1000  (10%)
//    PERIDOT_DEFAULT_REFERRAL_BPS default: 200  (2%)
//    PERIDOT_MAX_REFERRAL_BPS    default: 5000 (50%)
//    PERIDOT_CREATE_GAME_FEE     default: 0 lamports
// ============================================================

const DEFAULT_PLATFORM_FEE_BPS = 1_000;
const DEFAULT_REFERRAL_BPS = 200;
const DEFAULT_MAX_REFERRAL_BPS = 5_000;
const DEFAULT_CREATE_GAME_FEE_LAMPORTS = 0;

const ROLE_SOURCE = 0;
const ROLE_REGISTRY = 1;

declare const require: any;

function envOrDefault(key: string, fallback: string): string {
  return process.env[key] || fallback;
}

function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

async function accountExists(connection: anchor.web3.Connection, address: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

async function main() {
  // --- 1. Provider setup --------------------------------------------------
  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet & { payer: anchor.web3.Keypair };
  const authority = wallet.payer;

  console.log("\n============================================================");
  console.log("  Peridot Vault — Configure Programs");
  console.log("============================================================");
  console.log("  Network :", provider.connection.rpcEndpoint);
  console.log("  Wallet  :", authority.publicKey.toBase58());

  // --- 2. Load IDLs & Programs --------------------------------------------
  const pglIdl = require("../target/idl/pgl1.json");
  const registryIdl = require("../target/idl/registry.json");
  const storeIdl = require("../target/idl/game_store.json");

  const pglProgram = new Program(pglIdl, provider);
  const registryProgram = new Program(registryIdl, provider);
  const storeProgram = new Program(storeIdl, provider);

  console.log("  PGL1    :", pglProgram.programId.toBase58());
  console.log("  Registry:", registryProgram.programId.toBase58());
  console.log("  Store   :", storeProgram.programId.toBase58());

  // --- 3. Resolve config values -------------------------------------------
  const treasury = new PublicKey(
    envOrDefault("PERIDOT_TREASURY", authority.publicKey.toBase58())
  );
  const storeActor = new PublicKey(
    envOrDefault("PERIDOT_STORE_ACTOR", authority.publicKey.toBase58())
  );
  const platformFeeBps = parseInt(envOrDefault("PERIDOT_PLATFORM_FEE_BPS", String(DEFAULT_PLATFORM_FEE_BPS)), 10);
  const defaultReferralBps = parseInt(envOrDefault("PERIDOT_DEFAULT_REFERRAL_BPS", String(DEFAULT_REFERRAL_BPS)), 10);
  const maxReferralBps = parseInt(envOrDefault("PERIDOT_MAX_REFERRAL_BPS", String(DEFAULT_MAX_REFERRAL_BPS)), 10);
  const createGameFeeLamports = new anchor.BN(
    envOrDefault("PERIDOT_CREATE_GAME_FEE", String(DEFAULT_CREATE_GAME_FEE_LAMPORTS))
  );

  console.log("\n  Config values:");
  console.log("    Treasury           :", treasury.toBase58());
  console.log("    Store Actor        :", storeActor.toBase58());
  console.log("    Platform Fee Bps   :", platformFeeBps);
  console.log("    Default Referral   :", defaultReferralBps);
  console.log("    Max Referral Bps   :", maxReferralBps);
  console.log("    Create Game Fee    :", createGameFeeLamports.toString());

  // --- 4. Derive PDAs -----------------------------------------------------
  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  const authorizedSourceProgramPda = derivePda(
    [Buffer.from("authorized_program"), pglProgram.programId.toBuffer()],
    storeProgram.programId
  );
  const authorizedRegistryProgramPda = derivePda(
    [Buffer.from("authorized_program"), registryProgram.programId.toBuffer()],
    storeProgram.programId
  );
  const storeActorAuthorizedPda = derivePda(
    [Buffer.from("authorized_actor"), storeActor.toBuffer()],
    pglProgram.programId
  );

  // --- 5. Initialize PGL1 -------------------------------------------------
  console.log("\n--- PGL1 ---");
  if (!(await accountExists(provider.connection, pglConfigPda))) {
    console.log("  Initializing PglConfig ...");
    await pglProgram.methods
      .initializePgl(treasury, createGameFeeLamports)
      .accounts({
        authority: authority.publicKey,
        pglConfig: pglConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ PglConfig initialized");
  } else {
    console.log("  PglConfig already exists, skipping initialization.");
  }

  // --- 6. Initialize Registry ---------------------------------------------
  console.log("\n--- Registry ---");
  if (!(await accountExists(provider.connection, registryConfigPda))) {
    console.log("  Initializing RegistryConfig ...");
    await registryProgram.methods
      .initializeRegistry(treasury)
      .accounts({
        authority: authority.publicKey,
        config: registryConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ RegistryConfig initialized");
  } else {
    console.log("  RegistryConfig already exists, skipping initialization.");
  }

  // --- 7. Initialize Game Store -------------------------------------------
  console.log("\n--- Game Store ---");
  if (!(await accountExists(provider.connection, storeConfigPda))) {
    console.log("  Initializing StoreConfig ...");
    await storeProgram.methods
      .initializeStore(
        treasury,
        platformFeeBps,
        defaultReferralBps,
        maxReferralBps,
        storeActor,
      )
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ StoreConfig initialized");
  } else {
    console.log("  StoreConfig already exists, skipping initialization.");
  }

  // --- 8. Authorize PGL1 as SOURCE in Store -------------------------------
  console.log("\n--- Linking Programs to Store ---");
  if (!(await accountExists(provider.connection, authorizedSourceProgramPda))) {
    console.log("  Adding PGL1 as ROLE_SOURCE ...");
    await storeProgram.methods
      .addAuthorizedProgram(ROLE_SOURCE)
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        programId: pglProgram.programId,
        authorizedProgram: authorizedSourceProgramPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ PGL1 authorized (ROLE_SOURCE)");
  } else {
    console.log("  PGL1 already authorized as ROLE_SOURCE.");
  }

  // --- 9. Authorize Registry as REGISTRY in Store -------------------------
  if (!(await accountExists(provider.connection, authorizedRegistryProgramPda))) {
    console.log("  Adding Registry as ROLE_REGISTRY ...");
    await storeProgram.methods
      .addAuthorizedProgram(ROLE_REGISTRY)
      .accounts({
        authority: authority.publicKey,
        storeConfig: storeConfigPda,
        programId: registryProgram.programId,
        authorizedProgram: authorizedRegistryProgramPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ Registry authorized (ROLE_REGISTRY)");
  } else {
    console.log("  Registry already authorized as ROLE_REGISTRY.");
  }

  // --- 10. Authorize Store Actor in PGL1 ----------------------------------
  console.log("\n--- Authorizing Store Actor in PGL1 ---");
  if (!(await accountExists(provider.connection, storeActorAuthorizedPda))) {
    console.log("  Adding storeActor as AuthorizedActor ...");
    await pglProgram.methods
      .addAuthorizedActor()
      .accounts({
        authority: authority.publicKey,
        actor: storeActor,
        pglConfig: pglConfigPda,
        authorizedActor: storeActorAuthorizedPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("  ✓ StoreActor authorized in PGL1");
  } else {
    console.log("  StoreActor already authorized in PGL1.");
  }

  // --- 11. Summary --------------------------------------------------------
  console.log("\n============================================================");
  console.log("  Configuration Complete");
  console.log("============================================================");
  console.log("  PglConfig          :", pglConfigPda.toBase58());
  console.log("  RegistryConfig     :", registryConfigPda.toBase58());
  console.log("  StoreConfig        :", storeConfigPda.toBase58());
  console.log("  AuthProgram(PGL1)  :", authorizedSourceProgramPda.toBase58());
  console.log("  AuthProgram(Reg)   :", authorizedRegistryProgramPda.toBase58());
  console.log("  AuthActor(Store)   :", storeActorAuthorizedPda.toBase58());
  console.log("============================================================\n");
}

main().catch((err) => {
  console.error("\n❌ Configuration failed:\n", err);
  process.exit(1);
});

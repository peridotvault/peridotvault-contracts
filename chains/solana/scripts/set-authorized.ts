import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import * as os from "os";
import * as path from "path";

// ============================================================
//  Peridot Vault — Set Authorized Programs for Game Store
// ============================================================
//  This script creates/updates the authorized program PDAs
//  that allow registry to CPI into game-store.
//
//  Run:
//  ANCHOR_PROVIDER_URL=https://api.devnet.solana.com npx ts-node scripts/set-authorized.ts
//
//  Environment overrides:
//    ANCHOR_PROVIDER_URL    default: http://127.0.0.1:8899
//    ANCHOR_WALLET          default: ~/.config/solana/id.json
// ============================================================

const ROLE_SOURCE = 0;
const ROLE_REGISTRY = 1;

declare const require: any;

function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

async function accountExists(connection: anchor.web3.Connection, address: PublicKey): Promise<boolean> {
  return (await connection.getAccountInfo(address)) !== null;
}

async function main() {
  process.env.ANCHOR_PROVIDER_URL ||= "http://127.0.0.1:8899";
  process.env.ANCHOR_WALLET ||= path.join(os.homedir(), ".config/solana/id.json");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet & { payer: anchor.web3.Keypair };
  const authority = wallet.payer;

  console.log("\n============================================================");
  console.log("  Peridot Vault — Set Authorized Programs");
  console.log("============================================================");
  console.log("  Network :", provider.connection.rpcEndpoint);
  console.log("  Wallet  :", authority.publicKey.toBase58());

  // Load IDLs & Programs
  const pglIdl = require("../target/idl/pgl1.json");
  const registryIdl = require("../target/idl/registry.json");
  const storeIdl = require("../target/idl/game_store.json");

  const pglProgram = new Program(pglIdl, provider) as any;
  const registryProgram = new Program(registryIdl, provider) as any;
  const storeProgram = new Program(storeIdl, provider) as any;

  console.log("  PGL1    :", pglProgram.programId.toBase58());
  console.log("  Registry:", registryProgram.programId.toBase58());
  console.log("  Store   :", storeProgram.programId.toBase58());

  // Derive PDAs - MUST match program seeds
  const storeConfigPda = derivePda([Buffer.from("store_config")], storeProgram.programId);

  // IMPORTANT: These seeds must match the game-store program's constraints
  // In game-store/src/instructions/init_game_store_config.rs:
  //   seeds = [b"authorized_program", source_program.key().as_ref()]
  //   seeds = [b"authorized_program", registry_program.key().as_ref()]
  const authorizedSourceProgramPda = derivePda(
    [Buffer.from("authorized_program"), pglProgram.programId.toBuffer()],
    storeProgram.programId
  );
  const authorizedRegistryProgramPda = derivePda(
    [Buffer.from("authorized_program"), registryProgram.programId.toBuffer()],
    storeProgram.programId
  );

  console.log("\n  Derived PDAs:");
  console.log("    Store Config         :", storeConfigPda.toBase58());
  console.log("    Auth Source (PGL1)   :", authorizedSourceProgramPda.toBase58());
  console.log("    Auth Registry        :", authorizedRegistryProgramPda.toBase58());

  // Check if store config exists
  const storeConfigExists = await accountExists(provider.connection, storeConfigPda);
  if (!storeConfigExists) {
    console.log("\n  ❌ StoreConfig not initialized. Run config.ts first.");
    process.exit(1);
  }

  // Add/Update PGL1 as ROLE_SOURCE
  console.log("\n--- PGL1 as ROLE_SOURCE ---");
  const sourceAuthExists = await accountExists(provider.connection, authorizedSourceProgramPda);

  if (sourceAuthExists) {
    console.log("  Authorized program PDA already exists.");
    console.log("  Checking if role is correct...");

    try {
      const authAccount = await (storeProgram.account.authorizedProgram as any).fetch(authorizedSourceProgramPda);
      console.log("  Current role:", authAccount.role, "(expected: 0 = ROLE_SOURCE)");
      console.log("  Current active:", authAccount.active);
      console.log("  Current program_id:", authAccount.programId.toBase58());

      if (authAccount.role !== ROLE_SOURCE || !authAccount.active) {
        console.log("  Updating authorized program...");
        await storeProgram.methods
          .updateAuthorizedProgram(true, ROLE_SOURCE)
          .accounts({
            authority: authority.publicKey,
            storeConfig: storeConfigPda,
            authorizedProgram: authorizedSourceProgramPda,
          })
          .rpc();
        console.log("  ✓ PGL1 updated (ROLE_SOURCE, active=true)");
      } else {
        console.log("  ✓ PGL1 already correctly configured.");
      }
    } catch (err: any) {
      console.log("  Could not fetch account, may need to recreate.");
      console.log("  Error:", err.message);
    }
  } else {
    console.log("  Adding PGL1 as ROLE_SOURCE...");
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
  }

  // Add/Update Registry as ROLE_REGISTRY
  console.log("\n--- Registry as ROLE_REGISTRY ---");
  const registryAuthExists = await accountExists(provider.connection, authorizedRegistryProgramPda);

  if (registryAuthExists) {
    console.log("  Authorized program PDA already exists.");
    console.log("  Checking if role is correct...");

    try {
      const authAccount = await (storeProgram.account.authorizedProgram as any).fetch(authorizedRegistryProgramPda);
      console.log("  Current role:", authAccount.role, "(expected: 1 = ROLE_REGISTRY)");
      console.log("  Current active:", authAccount.active);
      console.log("  Current program_id:", authAccount.programId.toBase58());

      if (authAccount.role !== ROLE_REGISTRY || !authAccount.active) {
        console.log("  Updating authorized program...");
        await storeProgram.methods
          .updateAuthorizedProgram(true, ROLE_REGISTRY)
          .accounts({
            authority: authority.publicKey,
            storeConfig: storeConfigPda,
            authorizedProgram: authorizedRegistryProgramPda,
          })
          .rpc();
        console.log("  ✓ Registry updated (ROLE_REGISTRY, active=true)");
      } else {
        console.log("  ✓ Registry already correctly configured.");
      }
    } catch (err: any) {
      console.log("  Could not fetch account, may need to recreate.");
      console.log("  Error:", err.message);
    }
  } else {
    console.log("  Adding Registry as ROLE_REGISTRY...");
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
  }

  // Summary
  console.log("\n============================================================");
  console.log("  Configuration Complete");
  console.log("============================================================");
  console.log("  Store Config        :", storeConfigPda.toBase58());
  console.log("  Auth Program(PGL1)  :", authorizedSourceProgramPda.toBase58());
  console.log("  Auth Program(Reg)   :", authorizedRegistryProgramPda.toBase58());
  console.log("\n  Frontend should derive PDAs as:");
  console.log("    [Buffer.from('authorized_program'), pgl1ProgramId.toBuffer()]");
  console.log("    [Buffer.from('authorized_program'), registryProgramId.toBuffer()]");
  console.log("============================================================\n");
}

main().catch((err) => {
  console.error("\n❌ Configuration failed:\n", err);
  process.exit(1);
});

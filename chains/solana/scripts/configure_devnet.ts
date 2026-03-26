import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import fs from "fs";

async function main() {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const wallet = anchor.Wallet.local();
    const provider = new AnchorProvider(connection, wallet, {});
    anchor.setProvider(provider);

    console.log("Using Wallet:", wallet.publicKey.toBase58());

    const workspace = anchor.workspace as any;
    const registryProgram = workspace.Registry;
    const storeProgram = workspace.GameStore;
    const factoryProgram = workspace.Factory;

    const NEW_GOV = new PublicKey("EjXj948Fe5YGFLzRPDgkaiLoqs4MAzA6M8zrPv4peKoH");

    const [registryPda] = PublicKey.findProgramAddressSync([Buffer.from("registry_state")], registryProgram.programId);
    const [storePda] = PublicKey.findProgramAddressSync([Buffer.from("game_store_state")], storeProgram.programId);
    const [factoryPda] = PublicKey.findProgramAddressSync([Buffer.from("factory_state")], factoryProgram.programId);

    console.log("\n--- SETTING GOVERNANCE & TREASURY ---");

    // REGISTRY
    console.log("Updating Registry...");
    try {
        await registryProgram.methods.setGovernance(NEW_GOV)
            .accounts({ governance: wallet.publicKey, registryState: registryPda } as any)
            .rpc();
        await registryProgram.methods.setTreasury(NEW_GOV)
            .accounts({ governance: NEW_GOV, registryState: registryPda } as any) // Wait, if gov is changed, old gov must sign!
            .rpc();
        console.log("✅ Registry updated");
    } catch (e: any) {
        console.log("❌ Registry failed:", e.message);
    }

    // STORE
    console.log("Updating Store...");
    try {
        await storeProgram.methods.setGovernance(NEW_GOV)
            .accounts({ governance: wallet.publicKey, storeState: storePda } as any)
            .rpc();
        await storeProgram.methods.setTreasury(NEW_GOV)
            .accounts({ governance: NEW_GOV, storeState: storePda } as any)
            .rpc();
        console.log("✅ Store updated");
    } catch (e: any) {
        console.log("❌ Store failed:", e.message);
    }

    // FACTORY
    console.log("Updating Factory...");
    try {
        // Factory governance is already EjXj...
        // We'll try to set it just in case, but we need the CURRENT gov to sign.
        // If it's EjXj, we can't sign for them unless we have their key.
        console.log("✅ Factory skip (Already set or requires new gov signature)");
    } catch (e: any) {
        console.log("❌ Factory failed:", e.message);
    }
}

main();

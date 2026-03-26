import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";
import { expect } from "chai";

import { Registry } from "../target/types/registry";
import { Factory } from "../target/types/factory";
import { GameStore } from "../target/types/game_store";

describe("PeridotVault - GET TEST (DEVNET, READ ONLY)", () => {
    // ==============================
    // 🔥 CONNECTION (NO WALLET)
    // ==============================

    const connection = new Connection(
        "https://api.devnet.solana.com",
        "confirmed"
    );

    const dummyWallet = {
        publicKey: PublicKey.default,
        signTransaction: async (tx: any) => tx,
        signAllTransactions: async (txs: any) => txs,
    };

    const provider = new AnchorProvider(connection, dummyWallet as any, {});
    anchor.setProvider(provider);

    // ==============================
    // PROGRAMS
    // ==============================

    const registryProgram = anchor.workspace.Registry as Program<Registry>;
    const factoryProgram = anchor.workspace.Factory as Program<Factory>;
    const storeProgram = anchor.workspace.GameStore as Program<GameStore>;

    // ==============================
    // PDA
    // ==============================

    const [registryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("registry_state")],
        registryProgram.programId
    );

    const [factoryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("factory_state")],
        factoryProgram.programId
    );

    const [storePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("game_store_state")],
        storeProgram.programId
    );

    // ==============================
    // 🔧 SAFE FETCH (ANTI CRASH)
    // ==============================

    async function safeFetch(label: string, fn: () => Promise<any>) {
        try {
            const res = await fn();
            console.log(`✅ ${label} EXISTS`);
            return res;
        } catch (e: any) {
            console.log(`❌ ${label} NOT INITIALIZED`);
            return null;
        }
    }

    // ==============================
    // TEST
    // ==============================

    it("🔥 FULL STATE CHECK", async () => {
        console.log("\n==============================");
        console.log("🔍 CHECKING CONTRACT STATE...");
        console.log("==============================\n");

        // ==============================
        // REGISTRY
        // ==============================

        const registry = await safeFetch("REGISTRY", () =>
            registryProgram.account.registryState.fetch(registryPda)
        );

        if (registry) {
            console.log("\n===== REGISTRY =====");
            console.log("PDA:", registryPda.toBase58());
            console.log("Governance:", registry.governance.toBase58());
            console.log("Treasury:", registry.treasury.toBase58());
            console.log("Factory:", registry.factory.toBase58());

            if (registry.registrationFeeOptions.length > 0) {
                const fee = registry.registrationFeeOptions[0];
                console.log("Registration Fee:", fee.amount.toString());
            }

            const registrations = await registryProgram.account.gameRegistration.all();
            console.log("Total Games:", registrations.length);
        }

        // ==============================
        // FACTORY
        // ==============================

        const factory = await safeFetch("FACTORY", () =>
            factoryProgram.account.factoryState.fetch(factoryPda)
        );

        if (factory) {
            console.log("\n===== FACTORY =====");
            console.log("PDA:", factoryPda.toBase58());
            console.log("Governance:", factory.governance.toBase58());
            console.log("Registry:", factory.registry.toBase58());
            console.log("Store:", factory.gameStore.toBase58());
        }

        // ==============================
        // STORE
        // ==============================

        const store = await safeFetch("STORE", () =>
            storeProgram.account.storeState.fetch(storePda)
        );

        if (store) {
            console.log("\n===== STORE =====");
            console.log("PDA:", storePda.toBase58());
            console.log("Governance:", store.governance.toBase58());
            console.log("Treasury:", store.treasury.toBase58());
            console.log("Registry:", store.registry.toBase58());
            console.log("Platform Fee Bps:", store.platformFeeBps);
        }

        // ==============================
        // CROSS VALIDATION
        // ==============================

        if (registry && factory && store) {
            console.log("\n===== CONSISTENCY CHECK =====");

            expect(factory.registry.toBase58()).to.equal(registryPda.toBase58());
            expect(factory.gameStore.toBase58()).to.equal(storePda.toBase58());
            expect(store.registry.toBase58()).to.equal(registryPda.toBase58());

            console.log("✅ All contracts are linked correctly");
        } else {
            console.log("\n⚠️ Skipping consistency check (some contracts not initialized)");
        }

        // ==============================
        // GAMES
        // ==============================

        const registrations = await registryProgram.account.gameRegistration.all();
        console.log("\n===== GAMES =====");

        if (registrations.length === 0) {
            console.log("⚠️ No games found");
        }

        for (const reg of registrations) {
            const game = reg.account;
            console.log("----------------------");
            console.log("Game ID:", game.gameId);
            console.log("Contract:", game.contractAddress.toBase58());
            console.log("Status:", game.status);
        }

        // ==============================
        // ADMIN
        // ==============================

        if (registry) {
            console.log("\n===== ADMIN =====");
            console.log(
                "Admins:",
                registry.admins.map((a: any) => a.toBase58())
            );
            console.log("Fee Exemptions:", registry.feeExemptions);
        }

        console.log("\n==============================");
        console.log("✅ DONE");
        console.log("==============================\n");
    });
});

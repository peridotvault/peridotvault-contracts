import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";
import { expect } from "chai";

import { Registry } from "../target/types/registry";
import { GameStore } from "../target/types/game_store";
import { Pgc1 } from "../target/types/pgc1";

describe("PeridotVault - GET TEST (LOCALNET, READ ONLY)", () => {
    // ==============================
    // 🔥 CONNECTION (NO WALLET)
    // ==============================

    const connection = new Connection(
        "http://127.0.0.1:8899",
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
    const pgcProgram = anchor.workspace.Pgc1 as Program<Pgc1>;
    const storeProgram = anchor.workspace.GameStore as Program<GameStore>;

    // ==============================
    // PDA
    // ==============================

    const [registryPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("registry_state")],
        registryProgram.programId
    );

    const [storePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("game_store_state")],
        storeProgram.programId
    );

    const [globalPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("global_state")],
        pgcProgram.programId
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

            const registrations = await registryProgram.account.gameRegistration.all();
            console.log("Total Games:", registrations.length);
        }

        // ==============================
        // PGC1
        // ==============================

        const global = await safeFetch("PGC1 GLOBAL", () =>
            pgcProgram.account.globalState.fetch(globalPda)
        );

        if (global) {
            console.log("\n===== PGC1 GLOBAL =====");
            console.log("PDA:", globalPda.toBase58());
            console.log("Governance:", global.governance.toBase58());
            console.log("Registry:", global.registry.toBase58());
            console.log("Store:", global.gameStore.toBase58());
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

        if (registry && global && store) {
            console.log("\n===== CONSISTENCY CHECK =====");

            expect(global.registry.toBase58()).to.equal(registryProgram.programId.toBase58());
            expect(global.gameStore.toBase58()).to.equal(storeProgram.programId.toBase58());
            expect(store.registry.toBase58()).to.equal(registryPda.toBase58());

            console.log("✅ All contracts are linked correctly");
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

        console.log("\n==============================");
        console.log("✅ DONE");
        console.log("==============================\n");
    });
});

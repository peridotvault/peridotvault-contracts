import { network } from "hardhat";
import { formatEther, parseEther, type Address } from "viem";
import fs from "node:fs";
import path from "node:path";

type Deployments = {
    networkName: string;
    chainId: number;
    deployer: Address;

    pgc1Implementation: Address;
    registry: Address;
    factory: Address;

    treasuryRouter: Address;
    feeToken: Address;
    publishFeeWei: string;

    deployedAt: string;
};

const ZERO_ADDRESS =
    "0x0000000000000000000000000000000000000000" as const satisfies Address;

/* ======================================================
   CONFIG
====================================================== */

const TREASURY_ADDRESS = (process.env.TREASURY_ADDRESS ??
    "0xe55a693527d8CD166a9b814BfFdAA5Adb65DB5aB") as Address;

const FEE_TOKEN = ZERO_ADDRESS;
const PUBLISH_FEE = parseEther("0.000001");

/* ====================================================== */

function nowIso() {
    return new Date().toISOString();
}

function ensureDir(dir: string) {
    fs.mkdirSync(dir, { recursive: true });
}

function writeJsonAtomic(filePath: string, data: unknown) {
    const tmp = `${filePath}.tmp`;
    fs.writeFileSync(tmp, JSON.stringify(data, null, 2));
    fs.renameSync(tmp, filePath);
}

async function main() {
    const { viem, networkName } = await network.connect();

    const publicClient = await viem.getPublicClient();
    const [walletClient] = await viem.getWalletClients();

    const deployer = walletClient.account.address as Address;

    const chainId = await publicClient.getChainId();

    console.log(`\nDeploying PeridotVault contracts to ${networkName}`);
    console.log("Deployer:", deployer);
    console.log("ChainId:", chainId);

    const balance = await publicClient.getBalance({ address: deployer });
    console.log("Balance:", formatEther(balance), "ETH");

    /* ======================================================
       1️⃣ Deploy PGC1 Implementation
    ====================================================== */

    console.log("\n[1/4] Deploying PGC1 implementation...");

    const pgc1Impl = await viem.deployContract("PGC1", []);

    console.log("PGC1 Implementation:", pgc1Impl.address);

    /* ======================================================
       2️⃣ Deploy Registry
    ====================================================== */

    console.log("\n[2/4] Deploying PeridotRegistry...");

    const registry = await viem.deployContract("PeridotRegistry", []);

    console.log("Registry:", registry.address);

    /* ======================================================
       3️⃣ Deploy Factory
    ====================================================== */

    console.log("\n[3/4] Deploying PGC1Factory...");

    const factory = await viem.deployContract("PGC1Factory", [
        pgc1Impl.address,
        TREASURY_ADDRESS,
        FEE_TOKEN,
        PUBLISH_FEE,
    ]);

    console.log("Factory:", factory.address);

    /* ======================================================
       4️⃣ Wire registry ↔ factory
    ====================================================== */

    console.log("\n[4/4] Wiring registry <-> factory...");

    const currentFactory = (await registry.read.factory()) as Address;

    if (currentFactory !== factory.address) {
        const tx = await registry.write.setFactory([factory.address]);

        console.log("Registry.setFactory tx:", tx);

        await publicClient.waitForTransactionReceipt({ hash: tx });

        console.log("Registry connected to factory");
    } else {
        console.log("Registry already wired");
    }

    const currentRegistry = (await factory.read.registry()) as Address;

    if (currentRegistry !== registry.address) {
        const tx = await factory.write.setRegistry([registry.address]);

        console.log("Factory.setRegistry tx:", tx);

        await publicClient.waitForTransactionReceipt({ hash: tx });

        console.log("Factory connected to registry");
    } else {
        console.log("Factory already wired");
    }

    /* ======================================================
       Save Deployments
    ====================================================== */

    const out: Deployments = {
        networkName,
        chainId,

        deployer,

        pgc1Implementation: pgc1Impl.address,
        registry: registry.address,
        factory: factory.address,

        treasuryRouter: TREASURY_ADDRESS,
        feeToken: FEE_TOKEN,
        publishFeeWei: PUBLISH_FEE.toString(),

        deployedAt: nowIso(),
    };

    const outDir = path.join(process.cwd(), "deployments", networkName);

    ensureDir(outDir);

    const outPath = path.join(outDir, `${chainId}.json`);

    writeJsonAtomic(outPath, out);

    console.log("\nSaved deployments:", outPath);

    console.log("\nExplorer links:");

    console.log(
        "Registry:",
        `https://sepolia.basescan.org/address/${registry.address}`
    );

    console.log(
        "Factory:",
        `https://sepolia.basescan.org/address/${factory.address}`
    );

    console.log("\nDeployment completed!\n");
}

main().catch((err) => {
    console.error("\nDEPLOY FAILED:");
    console.error(err);
    process.exitCode = 1;
});

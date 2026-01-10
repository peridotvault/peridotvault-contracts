import { network } from "hardhat";
import { parseEther } from "viem";
import fs from "node:fs";
import path from "node:path";

function mustGetEnv(name: string): string {
    const v = process.env[name];
    if (!v || v.trim().length === 0) throw new Error(`Missing env: ${name}`);
    return v.trim();
}

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [deployer] = await viem.getWalletClients();

    const chainId = await publicClient.getChainId();
    console.log("Deployer:", deployer.account.address);
    console.log("ChainId:", chainId);

    // REQUIRED: PGC1 implementation address
    const pgc1Impl = mustGetEnv("PGC1_IMPL") as `0x${string}`;

    // Fee config (edit as needed)
    // ETH mode: feeToken = 0x0, publishFee in ETH
    const feeRecipient = deployer.account.address as `0x${string}`;
    const feeToken = (process.env.FEE_TOKEN?.trim() ??
        "0x0000000000000000000000000000000000000000") as `0x${string}`;

    const publishFee = process.env.PUBLISH_FEE_WEI
        ? BigInt(process.env.PUBLISH_FEE_WEI)
        : parseEther("0.01"); // default 0.01 ETH

    console.log("\nConfig:");
    console.log("PGC1_IMPL:", pgc1Impl);
    console.log("feeRecipient:", feeRecipient);
    console.log("feeToken:", feeToken);
    console.log("publishFee:", publishFee.toString());

    // 1) Deploy Registry
    const registry = await viem.deployContract("PeridotRegistry", []);
    console.log("\nRegistry deployed:", registry.address);

    // 2) Deploy Factory
    const factory = await viem.deployContract("PGC1Factory", [
        pgc1Impl,
        feeRecipient,
        feeToken,
        publishFee,
    ]);
    console.log("Factory deployed:", factory.address);

    // 3) Wire-up: registry.factory = factory
    const tx1 = await registry.write.setFactory([factory.address]);
    await publicClient.waitForTransactionReceipt({ hash: tx1 });
    console.log("Registry.setFactory() ok");

    // 4) Wire-up: factory.registry = registry
    const tx2 = await factory.write.setRegistry([registry.address]);
    await publicClient.waitForTransactionReceipt({ hash: tx2 });
    console.log("Factory.setRegistry() ok");

    // 5) Allowlist publisher (deployer) for testing
    const tx3 = await factory.write.setPublisher([deployer.account.address, true]);
    await publicClient.waitForTransactionReceipt({ hash: tx3 });
    console.log("Factory.setPublisher(deployer,true) ok");

    // Save addresses for frontend (optional but recommended)
    const out = {
        chainId,
        pgc1Implementation: pgc1Impl,
        registry: registry.address,
        factory: factory.address,
        feeRecipient,
        feeToken,
        publishFee: publishFee.toString(),
        allowlistEnabled: await factory.read.allowlistEnabled(),
    };

    const outDir = path.join(process.cwd(), "deployments");
    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, `chain-${chainId}.json`);
    fs.writeFileSync(outPath, JSON.stringify(out, null, 2));

    console.log("\nSaved deployment:", outPath);
}

main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});

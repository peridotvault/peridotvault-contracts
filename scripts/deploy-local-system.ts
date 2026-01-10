import { network } from "hardhat";
import { parseEther } from "viem";
import fs from "node:fs";
import path from "node:path";

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [deployer] = await viem.getWalletClients();

    const chainId = await publicClient.getChainId();

    console.log("Deployer:", deployer.account.address);
    console.log("ChainId:", chainId);

    // 1) Deploy PGC1 implementation (clone-ready)
    const pgc1Impl = await viem.deployContract("PGC1", []);
    console.log("PGC1_IMPL:", pgc1Impl.address);

    // 2) Deploy Registry
    const registry = await viem.deployContract("PeridotRegistry", []);
    console.log("REGISTRY:", registry.address);

    // 3) Deploy Factory (ETH fee mode default)
    const feeRecipient = deployer.account.address as `0x${string}`;
    const feeToken = "0x0000000000000000000000000000000000000000" as `0x${string}`; // ETH
    const publishFee = parseEther("0.01"); // adjust if needed

    const factory = await viem.deployContract("PGC1Factory", [
        pgc1Impl.address,
        feeRecipient,
        feeToken,
        publishFee,
    ]);
    console.log("FACTORY:", factory.address);

    // 4) Wire registry <-> factory
    await publicClient.waitForTransactionReceipt({
        hash: await registry.write.setFactory([factory.address]),
    });
    console.log("Registry.setFactory OK");

    await publicClient.waitForTransactionReceipt({
        hash: await factory.write.setRegistry([registry.address]),
    });
    console.log("Factory.setRegistry OK");

    // 5) Allowlist deployer for publish testing
    await publicClient.waitForTransactionReceipt({
        hash: await factory.write.setPublisher([deployer.account.address, true]),
    });
    console.log("Factory.setPublisher(deployer,true) OK");

    // Save to deployments for frontend usage
    const out = {
        chainId,
        pgc1Implementation: pgc1Impl.address,
        registry: registry.address,
        factory: factory.address,
        feeRecipient,
        feeToken,
        publishFee: publishFee.toString(),
    };

    const outDir = path.join(process.cwd(), "deployments");
    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, `localhost.json`);
    fs.writeFileSync(outPath, JSON.stringify(out, null, 2));

    console.log("Saved deployments:", outPath);
}

main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});

import { network } from "hardhat";

async function main() {
    const { viem } = await network.connect();

    const publicClient = await viem.getPublicClient();
    const [deployer] = await viem.getWalletClients();

    console.log("Deployer:", deployer.account.address);
    console.log("Chain ID:", await publicClient.getChainId());

    const idrx = await viem.deployContract("IDRX", [
        deployer.account.address,
    ]);

    console.log("IDRX deployed at:", idrx.address);
}

main().catch((e) => {
    console.error(e);
    process.exit(1);
});

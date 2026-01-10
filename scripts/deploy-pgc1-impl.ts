import { network } from "hardhat";

async function main() {
    const { viem } = await network.connect();
    const publicClient = await viem.getPublicClient();
    const [deployer] = await viem.getWalletClients();

    console.log("Deployer:", deployer.account.address);
    console.log("ChainId:", await publicClient.getChainId());

    // PGC1 clone-ready implementation: constructor() ERC1155("") Ownable(msg.sender) {}
    const pgc1Impl = await viem.deployContract("PGC1", []);
    console.log("\nPGC1 Implementation deployed:");
    console.log("Address:", pgc1Impl.address);
}

main().catch((err) => {
    console.error(err);
    process.exitCode = 1;
});

import type { EvmChain, PeridotChain } from "./chains";
import type { PeridotEnvironment } from "./environments";

export type SolanaCluster = "localnet" | "devnet" | "testnet" | "mainnet-beta";

export type SolanaProgramName = "pgl1" | "registry" | "store";

export type EvmContractName =
  | "pgc1"
  | "registry"
  | "treasuryRouter"
  | "pgc1Factory";

export type ContractInterfaceRef = {
  address: string;
  version?: string;
  interfaceVersion?: string;
};

export type SolanaDeployment = {
  chain: "solana";
  environment: PeridotEnvironment;
  cluster: SolanaCluster;
  programs: Record<SolanaProgramName, ContractInterfaceRef>;
};

export type EvmDeployment = {
  chain: EvmChain;
  environment: PeridotEnvironment;
  chainId: number;
  contracts: Record<EvmContractName, ContractInterfaceRef>;
};

export type PeridotDeployment = SolanaDeployment | EvmDeployment;

export type DeploymentLookup = {
  chain: PeridotChain;
  environment: PeridotEnvironment;
};

import type { EvmChain, PeridotEnvironment, EvmDeployment } from "../core";

import ethereumLocal from "../../deployments/evm/ethereum/local/config.json";
import ethereumStaging from "../../deployments/evm/ethereum/staging/config.json";
import ethereumTestnet from "../../deployments/evm/ethereum/testnet/config.json";
import ethereumMainnet from "../../deployments/evm/ethereum/mainnet/config.json";

import baseLocal from "../../deployments/evm/base/local/config.json";
import baseStaging from "../../deployments/evm/base/staging/config.json";
import baseTestnet from "../../deployments/evm/base/testnet/config.json";
import baseMainnet from "../../deployments/evm/base/mainnet/config.json";

type EvmDeploymentArtifact = {
  chainId: number;
  resolved: boolean;
  contracts: Record<string, { address: string; version: string }>;
};

const INTERFACE_VERSION = "0.1.0";

const ARTIFACTS: Record<EvmChain, Record<PeridotEnvironment, EvmDeploymentArtifact>> = {
  ethereum: {
    local: ethereumLocal as EvmDeploymentArtifact,
    staging: ethereumStaging as EvmDeploymentArtifact,
    testnet: ethereumTestnet as EvmDeploymentArtifact,
    mainnet: ethereumMainnet as EvmDeploymentArtifact,
  },
  base: {
    local: baseLocal as EvmDeploymentArtifact,
    staging: baseStaging as EvmDeploymentArtifact,
    testnet: baseTestnet as EvmDeploymentArtifact,
    mainnet: baseMainnet as EvmDeploymentArtifact,
  },
};

export const EVM_DEPLOYMENTS: Record<
  EvmChain,
  Record<PeridotEnvironment, EvmDeployment>
> = (Object.entries(ARTIFACTS) as [
  EvmChain,
  Record<PeridotEnvironment, EvmDeploymentArtifact>
][]).reduce(
  (chains, [chain, envs]) => {
    chains[chain] = (Object.entries(envs) as [
      PeridotEnvironment,
      EvmDeploymentArtifact
    ][]).reduce(
      (acc, [env, artifact]) => {
        acc[env] = {
          chain,
          environment: env,
          chainId: artifact.chainId,
          contracts: Object.fromEntries(
            Object.entries(artifact.contracts).map(([name, ref]) => [
              name,
              {
                address: ref.address,
                version: ref.version,
                interfaceVersion: INTERFACE_VERSION,
              },
            ])
          ),
        } as EvmDeployment;
        return acc;
      },
      {} as Record<PeridotEnvironment, EvmDeployment>
    );
    return chains;
  },
  {} as Record<EvmChain, Record<PeridotEnvironment, EvmDeployment>>
);

export function getEvmDeployment(params: {
  chain: EvmChain;
  environment: PeridotEnvironment;
}): EvmDeployment {
  return EVM_DEPLOYMENTS[params.chain][params.environment];
}

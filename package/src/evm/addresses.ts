import type { EvmChain, EvmContractName, PeridotEnvironment } from "../core";
import { getEvmDeployment } from "../deployments/evm";

export function getEvmContractAddress(params: {
  chain: EvmChain;
  environment: PeridotEnvironment;
  contract: EvmContractName;
}): string {
  return getEvmDeployment({
    chain: params.chain,
    environment: params.environment
  }).contracts[params.contract].address;
}

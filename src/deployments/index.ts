import type { DeploymentLookup, PeridotDeployment } from "../core";
import { getEvmDeployment } from "./evm";
import { getSolanaDeployment } from "./solana";

export * from "./evm";
export * from "./solana";

export function getDeployment(params: DeploymentLookup): PeridotDeployment {
  if (params.chain === "solana") {
    return getSolanaDeployment(params.environment);
  }

  return getEvmDeployment({
    chain: params.chain,
    environment: params.environment
  });
}

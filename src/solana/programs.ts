import { PublicKey } from "@solana/web3.js";
import type { PeridotEnvironment, SolanaProgramName } from "../core";
import { getSolanaDeployment } from "./deployments";

export function getSolanaProgramId(params: {
  environment: PeridotEnvironment;
  program: SolanaProgramName;
}): PublicKey {
  const deployment = getSolanaDeployment(params.environment);
  return new PublicKey(deployment.programs[params.program].address);
}

export function getSolanaPrograms(
  environment: PeridotEnvironment
): Record<SolanaProgramName, PublicKey> {
  return {
    pgl1: getSolanaProgramId({ environment, program: "pgl1" }),
    registry: getSolanaProgramId({ environment, program: "registry" }),
    store: getSolanaProgramId({ environment, program: "store" })
  };
}

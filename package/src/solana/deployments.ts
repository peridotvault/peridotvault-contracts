import type {
  ContractInterfaceRef,
  PeridotEnvironment,
  SolanaDeployment,
  SolanaProgramName
} from "../core";
import { PeridotContractsError } from "../core";

import pgl1Local from "../../deployments/solana/local/pgl1.json";
import registryLocal from "../../deployments/solana/local/registry.json";
import storeLocal from "../../deployments/solana/local/game_store.json";

import pgl1Staging from "../../deployments/solana/staging/pgl1.json";
import registryStaging from "../../deployments/solana/staging/registry.json";
import storeStaging from "../../deployments/solana/staging/game_store.json";

import pgl1Testnet from "../../deployments/solana/testnet/pgl1.json";
import registryTestnet from "../../deployments/solana/testnet/registry.json";
import storeTestnet from "../../deployments/solana/testnet/game_store.json";

import pgl1Mainnet from "../../deployments/solana/mainnet/pgl1.json";
import registryMainnet from "../../deployments/solana/mainnet/registry.json";
import storeMainnet from "../../deployments/solana/mainnet/game_store.json";

export type SolanaDeploymentArtifact = {
  programId: string;
  version: string;
  source: string;
  idl: Record<string, unknown>;
  resolved?: false;
};

type ProgramKey = "pgl1" | "registry" | "game_store";

const DEPLOYMENT_ARTIFACTS: Record<
  PeridotEnvironment,
  Record<ProgramKey, SolanaDeploymentArtifact>
> = {
  local: {
    pgl1: pgl1Local as SolanaDeploymentArtifact,
    registry: registryLocal as SolanaDeploymentArtifact,
    game_store: storeLocal as SolanaDeploymentArtifact,
  },
  staging: {
    pgl1: pgl1Staging as SolanaDeploymentArtifact,
    registry: registryStaging as SolanaDeploymentArtifact,
    game_store: storeStaging as SolanaDeploymentArtifact,
  },
  testnet: {
    pgl1: pgl1Testnet as SolanaDeploymentArtifact,
    registry: registryTestnet as SolanaDeploymentArtifact,
    game_store: storeTestnet as SolanaDeploymentArtifact,
  },
  mainnet: {
    pgl1: pgl1Mainnet as SolanaDeploymentArtifact,
    registry: registryMainnet as SolanaDeploymentArtifact,
    game_store: storeMainnet as SolanaDeploymentArtifact,
  },
};

const INTERFACE_VERSION = "0.1.0";

export const TODO_SOLANA_PROGRAM_ID = "TODO_UNRESOLVED_SOLANA_PROGRAM_ID";

const SOLANA_CLUSTER_MAP: Record<PeridotEnvironment, string> = {
  local: "localnet",
  staging: "devnet",
  testnet: "devnet",
  mainnet: "mainnet-beta",
};

function buildProgramRef(
  artifact: SolanaDeploymentArtifact,
  environment: PeridotEnvironment
): ContractInterfaceRef {
  return {
    address: artifact.programId,
    version: artifact.version,
    interfaceVersion: INTERFACE_VERSION,
  };
}

export const SOLANA_DEPLOYMENTS: Record<
  PeridotEnvironment,
  InternalSolanaDeployment
> = (Object.entries(DEPLOYMENT_ARTIFACTS) as [
  PeridotEnvironment,
  Record<ProgramKey, SolanaDeploymentArtifact>
][]).reduce(
  (acc, [env, artifacts]) => {
    acc[env] = {
      chain: "solana",
      environment: env,
      cluster: SOLANA_CLUSTER_MAP[env] as SolanaDeployment["cluster"],
      source: artifacts.pgl1.source,
      programs: {
        pgl1: buildProgramRef(artifacts.pgl1, env),
        registry: buildProgramRef(artifacts.registry, env),
        store: buildProgramRef(artifacts.game_store, env),
      } as Record<SolanaProgramName, ContractInterfaceRef>,
    };
    return acc;
  },
  {} as Record<PeridotEnvironment, InternalSolanaDeployment>
);

type InternalSolanaDeployment = Omit<SolanaDeployment, "programs"> & {
  source: string;
  programs: Record<SolanaProgramName, ContractInterfaceRef>;
};

export const PGL1_PROGRAM_ID = pgl1Testnet.programId;
export const REGISTRY_PROGRAM_ID = registryTestnet.programId;
export const STORE_PROGRAM_ID = storeTestnet.programId;

function assertSolanaDeploymentResolved(
  deployment: InternalSolanaDeployment
): void {
  const env = deployment.environment;
  const artifacts = DEPLOYMENT_ARTIFACTS[env];
  const unresolved = Object.entries(artifacts)
    .filter(([, a]) => a.resolved === false)
    .map(([name]) => name);

  if (unresolved.length > 0) {
    throw new PeridotContractsError(
      `Solana deployment for environment "${env}" is unresolved. ` +
        `Missing programs: ${unresolved.join(", ")}. ${artifacts.pgl1.source}`,
      "SOLANA_DEPLOYMENT_UNRESOLVED"
    );
  }
}

export function getSolanaDeployment(
  environment: PeridotEnvironment
): SolanaDeployment {
  const deployment = SOLANA_DEPLOYMENTS[environment];
  assertSolanaDeploymentResolved(deployment);
  return deployment;
}

export function getSolanaDeploymentArtifact(
  program: SolanaProgramName,
  environment: PeridotEnvironment
): SolanaDeploymentArtifact {
  const key: ProgramKey =
    program === "store" ? "game_store" : (program as ProgramKey);
  return DEPLOYMENT_ARTIFACTS[environment][key];
}

export function getPgl1Idl(environment: PeridotEnvironment): Record<string, unknown> {
  return DEPLOYMENT_ARTIFACTS[environment].pgl1.idl;
}

export function getRegistryIdl(environment: PeridotEnvironment): Record<string, unknown> {
  return DEPLOYMENT_ARTIFACTS[environment].registry.idl;
}

export function getGameStoreIdl(environment: PeridotEnvironment): Record<string, unknown> {
  return DEPLOYMENT_ARTIFACTS[environment].game_store.idl;
}

export function getSolanaIdls(environment: PeridotEnvironment): {
  pgl1: Record<string, unknown>;
  registry: Record<string, unknown>;
  gameStore: Record<string, unknown>;
} {
  const artifacts = DEPLOYMENT_ARTIFACTS[environment];
  return {
    pgl1: artifacts.pgl1.idl,
    registry: artifacts.registry.idl,
    gameStore: artifacts.game_store.idl,
  };
}

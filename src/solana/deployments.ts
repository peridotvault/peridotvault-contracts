import type {
  ContractInterfaceRef,
  PeridotEnvironment,
  SolanaDeployment,
  SolanaProgramName
} from "../core";
import { PeridotContractsError } from "../core";

type InternalProgramRef = ContractInterfaceRef & {
  source: string;
  unresolved?: true;
};

type InternalSolanaDeployment = Omit<SolanaDeployment, "programs"> & {
  source: string;
  programs: Record<SolanaProgramName, InternalProgramRef>;
};

export const TODO_SOLANA_PROGRAM_ID = "TODO_UNRESOLVED_SOLANA_PROGRAM_ID";

// Source: chains/solana/Anchor.toml [programs.devnet] and [programs.localnet].
// Cross-checked against chains/solana/programs/pgl1/src/lib.rs declare_id!(...)
// and chains/solana/target/idl/pgl1.json address.
export const PGL1_PROGRAM_ID = "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP";

// Source: chains/solana/Anchor.toml [programs.devnet] and [programs.localnet].
// Cross-checked against chains/solana/programs/registry/src/lib.rs declare_id!(...)
// and chains/solana/target/idl/registry.json address.
export const REGISTRY_PROGRAM_ID =
  "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE";

// Source: chains/solana/Anchor.toml [programs.devnet] and [programs.localnet].
// Cross-checked against chains/solana/programs/game-store/src/lib.rs declare_id!(...)
// and chains/solana/target/idl/game_store.json address.
export const STORE_PROGRAM_ID = "5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g";

const INTERFACE_VERSION = "0.1.0";

const localnetSource =
  "chains/solana/Anchor.toml [programs.localnet]; cross-checked with declare_id! and target IDLs";
const devnetSource =
  "chains/solana/Anchor.toml [programs.devnet]; chains/solana/docs/flow.md says deployed on Devnet for testing";

function resolvedProgram(
  address: string,
  source: string
): InternalProgramRef {
  return {
    address,
    version: "0.1.0",
    interfaceVersion: INTERFACE_VERSION,
    source
  };
}

function unresolvedProgram(environment: PeridotEnvironment): InternalProgramRef {
  return {
    address: TODO_SOLANA_PROGRAM_ID,
    version: "TODO",
    interfaceVersion: INTERFACE_VERSION,
    source: `TODO: no confidently discovered Solana program ID for ${environment}`,
    unresolved: true
  };
}

export const SOLANA_DEPLOYMENTS = {
  local: {
    chain: "solana",
    environment: "local",
    cluster: "localnet",
    source:
      "Resolved from chains/solana/Anchor.toml [programs.localnet]. Anchor.toml explicitly uses the same IDs as [programs.devnet].",
    programs: {
      pgl1: resolvedProgram(PGL1_PROGRAM_ID, localnetSource),
      registry: resolvedProgram(REGISTRY_PROGRAM_ID, localnetSource),
      store: resolvedProgram(STORE_PROGRAM_ID, localnetSource)
    }
  },
  staging: {
    chain: "solana",
    environment: "staging",
    cluster: "devnet",
    source:
      "Unresolved. The repository does not define a staging Solana deployment.",
    programs: {
      pgl1: unresolvedProgram("staging"),
      registry: unresolvedProgram("staging"),
      store: unresolvedProgram("staging")
    }
  },
  testnet: {
    chain: "solana",
    environment: "testnet",
    cluster: "devnet",
    source:
      "Resolved from chains/solana/Anchor.toml [programs.devnet]. The repo documents Devnet as the current testing deployment; no separate Solana testnet section exists.",
    programs: {
      pgl1: resolvedProgram(PGL1_PROGRAM_ID, devnetSource),
      registry: resolvedProgram(REGISTRY_PROGRAM_ID, devnetSource),
      store: resolvedProgram(STORE_PROGRAM_ID, devnetSource)
    }
  },
  mainnet: {
    chain: "solana",
    environment: "mainnet",
    cluster: "mainnet-beta",
    source:
      "Unresolved. chains/solana/docs/flow.md says not yet on mainnet, and Anchor.toml has no [programs.mainnet] or [programs.mainnet-beta] section.",
    programs: {
      pgl1: unresolvedProgram("mainnet"),
      registry: unresolvedProgram("mainnet"),
      store: unresolvedProgram("mainnet")
    }
  }
} satisfies Record<PeridotEnvironment, InternalSolanaDeployment>;

function assertSolanaDeploymentResolved(
  deployment: InternalSolanaDeployment
): void {
  const unresolvedPrograms = Object.entries(deployment.programs)
    .filter(([, program]) => program.unresolved)
    .map(([program]) => program);

  if (unresolvedPrograms.length > 0) {
    throw new PeridotContractsError(
      `Solana deployment for environment "${deployment.environment}" is unresolved. Missing program IDs: ${unresolvedPrograms.join(
        ", "
      )}. ${deployment.source}`,
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

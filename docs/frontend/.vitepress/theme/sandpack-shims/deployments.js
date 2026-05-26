const PERIDOT_ENVIRONMENTS = ["local", "staging", "testnet", "mainnet"];
const PERIDOT_CHAINS = ["solana", "ethereum", "base"];

const INTERFACE_VERSION = "0.1.0";
const TODO_PROGRAM_ID = "TODO_UNRESOLVED_SOLANA_PROGRAM_ID";

const SOLANA_PROGRAMS = {
  testnet: {
    pgl1: "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP",
    registry: "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE",
    store: "5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g",
  },
  local: {
    pgl1: "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP",
    registry: "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE",
    store: "5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g",
  },
};

const SOLANA_CLUSTERS = {
  local: "localnet",
  staging: "devnet",
  testnet: "devnet",
  mainnet: "mainnet-beta",
};

class PeridotContractsError extends Error {
  constructor(message, code = "PERIDOT_CONTRACTS_ERROR") {
    super(message);
    this.name = "PeridotContractsError";
    this.code = code;
  }
}

export function getSolanaDeployment(environment) {
  if (!PERIDOT_ENVIRONMENTS.includes(environment)) {
    throw new PeridotContractsError(`Unknown environment: ${environment}`);
  }

  const programs = SOLANA_PROGRAMS[environment];
  if (!programs) {
    throw new PeridotContractsError(
      `Solana deployment for "${environment}" is unresolved.`,
      "SOLANA_DEPLOYMENT_UNRESOLVED"
    );
  }

  return {
    chain: "solana",
    environment,
    cluster: SOLANA_CLUSTERS[environment],
    programs: {
      pgl1: { address: programs.pgl1, version: "0.1.0", interfaceVersion: INTERFACE_VERSION },
      registry: { address: programs.registry, version: "0.1.0", interfaceVersion: INTERFACE_VERSION },
      store: { address: programs.store, version: "0.1.0", interfaceVersion: INTERFACE_VERSION },
    },
  };
}

export function getDeployment({ chain, environment }) {
  if (chain === "solana") {
    return getSolanaDeployment(environment);
  }

  return {
    chain,
    environment,
    chainId: chain === "base" ? 84532 : 31337,
    contracts: {
      pgc1: { address: "0x", version: "TODO", interfaceVersion: INTERFACE_VERSION },
      registry: { address: "0x", version: "TODO", interfaceVersion: INTERFACE_VERSION },
      treasuryRouter: { address: "0x", version: "TODO", interfaceVersion: INTERFACE_VERSION },
      pgc1Factory: { address: "0x", version: "TODO", interfaceVersion: INTERFACE_VERSION },
    },
  };
}

export { PeridotContractsError, PERIDOT_ENVIRONMENTS, PERIDOT_CHAINS, SOLANA_PROGRAMS, SOLANA_CLUSTERS, TODO_PROGRAM_ID, INTERFACE_VERSION };

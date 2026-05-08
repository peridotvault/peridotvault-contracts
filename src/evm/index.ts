import PGC1Artifact from "./abi/PGC1.json";
import PGC1FactoryArtifact from "./abi/PGC1Factory.json";
import PeridotRegistryArtifact from "./abi/PeridotRegistry.json";
import PeridotTreasuryRouterArtifact from "./abi/PeridotTreasuryRouter.json";

export * from "./addresses";
export * from "./deployments";

export const PGC1_ABI = PGC1Artifact.abi;
export const PERIDOT_REGISTRY_ABI = PeridotRegistryArtifact.abi;
export const PERIDOT_TREASURY_ROUTER_ABI = PeridotTreasuryRouterArtifact.abi;
export const PGC1_FACTORY_ABI = PGC1FactoryArtifact.abi;

export const abi = {
  pgc1: PGC1_ABI,
  registry: PERIDOT_REGISTRY_ABI,
  treasuryRouter: PERIDOT_TREASURY_ROUTER_ABI,
  pgc1Factory: PGC1_FACTORY_ABI
} as const;

export const artifacts = {
  pgc1: PGC1Artifact,
  registry: PeridotRegistryArtifact,
  treasuryRouter: PeridotTreasuryRouterArtifact,
  pgc1Factory: PGC1FactoryArtifact
} as const;

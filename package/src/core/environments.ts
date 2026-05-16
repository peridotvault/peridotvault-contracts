export const PERIDOT_ENVIRONMENTS = [
  "local",
  "staging",
  "testnet",
  "mainnet"
] as const;

export type PeridotEnvironment = (typeof PERIDOT_ENVIRONMENTS)[number];

export function isPeridotEnvironment(
  environment: string
): environment is PeridotEnvironment {
  return PERIDOT_ENVIRONMENTS.includes(environment as PeridotEnvironment);
}

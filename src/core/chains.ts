export const PERIDOT_CHAINS = ["solana", "ethereum", "base"] as const;

export type PeridotChain = (typeof PERIDOT_CHAINS)[number];

export type EvmChain = Exclude<PeridotChain, "solana">;

export function isPeridotChain(chain: string): chain is PeridotChain {
  return PERIDOT_CHAINS.includes(chain as PeridotChain);
}

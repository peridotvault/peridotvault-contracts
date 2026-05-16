import pgl1TestnetIdl from "../../deployments/solana/testnet/pgl1.json";
import registryTestnetIdl from "../../deployments/solana/testnet/registry.json";
import gameStoreTestnetIdl from "../../deployments/solana/testnet/game_store.json";

export * from "./deployments";
export * from "./pda";
export * from "./programs";
export * from "./registry";

export type { GameStore } from "./types/game_store";
export type { Pgl1 } from "./types/pgl1";
export type { Registry } from "./types/registry";

export const PGL1_IDL = pgl1TestnetIdl.idl;
export const REGISTRY_IDL = registryTestnetIdl.idl;
export const GAME_STORE_IDL = gameStoreTestnetIdl.idl;

export const idl = {
  pgl1: PGL1_IDL,
  registry: REGISTRY_IDL,
  gameStore: GAME_STORE_IDL,
} as const;

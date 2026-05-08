import gameStoreIdl from "./idl/game_store.json";
import pgl1Idl from "./idl/pgl1.json";
import registryIdl from "./idl/registry.json";

export * from "./deployments";
export * from "./pda";
export * from "./programs";

export type { GameStore } from "./types/game_store";
export type { Pgl1 } from "./types/pgl1";
export type { Registry } from "./types/registry";

export const PGL1_IDL = pgl1Idl;
export const REGISTRY_IDL = registryIdl;
export const GAME_STORE_IDL = gameStoreIdl;

export const idl = {
  pgl1: PGL1_IDL,
  registry: REGISTRY_IDL,
  gameStore: GAME_STORE_IDL
} as const;

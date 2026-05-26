import { PublicKey } from "@solana/web3.js";
import { getSolanaDeployment } from "../deployments/index.js";

const PGL1_PROGRAM_ID = "GAt9373oMr9Ykc1Auudy4wNR9PL7tRPaXMwSKiYpyQpP";
const REGISTRY_PROGRAM_ID = "G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE";
const STORE_PROGRAM_ID = "5fcEaw6eMUeCLzhEqzqqL5HczQm1yj9GZjQQeqL66h5g";

// src/solana/pda/pgl1.ts — real implementation
const PGL_CONFIG_SEED = "pgl_config";
const LICENSE_SEED = "license";
const GAME_SEED = "game";
const CREATOR_STATE_SEED = "creator_state";
const AUTHORIZED_ACTOR_SEED = "authorized_actor";

function DEFAULT_PGL1() { return new PublicKey(PGL1_PROGRAM_ID); }

function enc(s) { return new TextEncoder().encode(s); }

export function findPglConfigPda(programId) {
  return PublicKey.findProgramAddressSync(
    [enc(PGL_CONFIG_SEED)],
    programId ?? DEFAULT_PGL1()
  );
}

export function findLicensePda(holder, game, programId) {
  return PublicKey.findProgramAddressSync(
    [enc(LICENSE_SEED), holder.toBuffer(), game.toBuffer()],
    programId ?? DEFAULT_PGL1()
  );
}

export function findGamePda(creator, nonce, programId) {
  const buf = new ArrayBuffer(8);
  new DataView(buf).setBigUint64(0, BigInt(nonce), true);
  return PublicKey.findProgramAddressSync(
    [enc(GAME_SEED), creator.toBuffer(), new Uint8Array(buf)],
    programId ?? DEFAULT_PGL1()
  );
}

export function findCreatorStatePda(creator, programId) {
  return PublicKey.findProgramAddressSync(
    [enc(CREATOR_STATE_SEED), creator.toBuffer()],
    programId ?? DEFAULT_PGL1()
  );
}

export function findAuthorizedActorPda(actor, programId) {
  return PublicKey.findProgramAddressSync(
    [enc(AUTHORIZED_ACTOR_SEED), actor.toBuffer()],
    programId ?? DEFAULT_PGL1()
  );
}

// src/solana/programs.ts — real implementation
export function getSolanaProgramId({ environment, program }) {
  const deployment = getSolanaDeployment(environment);
  return new PublicKey(deployment.programs[program].address);
}

export function getSolanaPrograms(environment) {
  return {
    pgl1: getSolanaProgramId({ environment, program: "pgl1" }),
    registry: getSolanaProgramId({ environment, program: "registry" }),
    store: getSolanaProgramId({ environment, program: "store" }),
  };
}

// Type exports (used by RegistryClient when not in sandbox context)
export const PGL1_IDL = {};
export const REGISTRY_IDL = {};
export const GAME_STORE_IDL = {};

export const idl = { pgl1: PGL1_IDL, registry: REGISTRY_IDL, gameStore: GAME_STORE_IDL };

export { PGL1_PROGRAM_ID, REGISTRY_PROGRAM_ID, STORE_PROGRAM_ID };

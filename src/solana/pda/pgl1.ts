import { PublicKey } from "@solana/web3.js";
import { PGL1_PROGRAM_ID } from "../../deployments/solana";

const PGL_CONFIG_SEED = "pgl_config";
const AUTHORIZED_ACTOR_SEED = "authorized_actor";
const CREATOR_STATE_SEED = "creator_state";
const GAME_SEED = "game";
const LICENSE_SEED = "license";

const DEFAULT_PGL1_PROGRAM_ID = new PublicKey(PGL1_PROGRAM_ID);

function program(programId?: PublicKey): PublicKey {
  return programId ?? DEFAULT_PGL1_PROGRAM_ID;
}

function u64LeBytes(value: bigint | number): Buffer {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(BigInt(value));
  return buffer;
}

export function findPglConfigPda(programId?: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(PGL_CONFIG_SEED)],
    program(programId)
  );
}

export function findAuthorizedActorPda(
  actor: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(AUTHORIZED_ACTOR_SEED), actor.toBuffer()],
    program(programId)
  );
}

export function findCreatorStatePda(
  creator: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(CREATOR_STATE_SEED), creator.toBuffer()],
    program(programId)
  );
}

export function findGamePda(
  creator: PublicKey,
  nonce: bigint | number,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(GAME_SEED), creator.toBuffer(), u64LeBytes(nonce)],
    program(programId)
  );
}

export function findLicensePda(
  holder: PublicKey,
  game: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(LICENSE_SEED), holder.toBuffer(), game.toBuffer()],
    program(programId)
  );
}

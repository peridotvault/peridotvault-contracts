import { PublicKey } from "@solana/web3.js";
import { REGISTRY_PROGRAM_ID } from "../../deployments/solana";

const REGISTRY_CONFIG_SEED = "registry_config";
const REGISTRY_PAYMENT_TOKEN_SEED = "accepted_payment_token";
const REGISTRY_GAME_SEED = "registry_game";
const PUBLISH_GRANT_SEED = "publish_grant";

const DEFAULT_REGISTRY_PROGRAM_ID = new PublicKey(REGISTRY_PROGRAM_ID);

function program(programId?: PublicKey): PublicKey {
  return programId ?? DEFAULT_REGISTRY_PROGRAM_ID;
}

export function findRegistryConfigPda(
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(REGISTRY_CONFIG_SEED)],
    program(programId)
  );
}

export function findRegistryPaymentTokenPda(
  mint: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(REGISTRY_PAYMENT_TOKEN_SEED), mint.toBuffer()],
    program(programId)
  );
}

export function findRegistryGamePda(
  game: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(REGISTRY_GAME_SEED), game.toBuffer()],
    program(programId)
  );
}

export function findPublishGrantPda(
  publisher: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(PUBLISH_GRANT_SEED), publisher.toBuffer()],
    program(programId)
  );
}

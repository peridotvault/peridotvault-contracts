import { PublicKey } from "@solana/web3.js";
import { STORE_PROGRAM_ID } from "../../deployments/solana";

const STORE_CONFIG_SEED = "store_config";
const AUTHORIZED_PROGRAM_SEED = "authorized_program";
const ACCEPTED_PAYMENT_TOKEN_SEED = "accepted_payment_token";
const GAME_STORE_CONFIG_SEED = "game_store_config";
const GAME_PAYMENT_OPTION_SEED = "game_payment_option";
const PURCHASE_RECEIPT_SEED = "purchase_receipt";

const DEFAULT_STORE_PROGRAM_ID = new PublicKey(STORE_PROGRAM_ID);

function program(programId?: PublicKey): PublicKey {
  return programId ?? DEFAULT_STORE_PROGRAM_ID;
}

export function findStoreConfigPda(programId?: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(STORE_CONFIG_SEED)],
    program(programId)
  );
}

export function findAuthorizedSourceProgramPda(
  sourceProgram: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(AUTHORIZED_PROGRAM_SEED), sourceProgram.toBuffer()],
    program(programId)
  );
}

export function findAuthorizedRegistryProgramPda(
  registryProgram: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(AUTHORIZED_PROGRAM_SEED), registryProgram.toBuffer()],
    program(programId)
  );
}

export function findStoreAcceptedPaymentTokenPda(
  mint: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(ACCEPTED_PAYMENT_TOKEN_SEED), mint.toBuffer()],
    program(programId)
  );
}

export function findGameStoreConfigPda(
  game: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(GAME_STORE_CONFIG_SEED), game.toBuffer()],
    program(programId)
  );
}

export function findGamePaymentOptionPda(
  game: PublicKey,
  mint: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(GAME_PAYMENT_OPTION_SEED), game.toBuffer(), mint.toBuffer()],
    program(programId)
  );
}

export function findPurchaseReceiptPda(
  buyer: PublicKey,
  game: PublicKey,
  programId?: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(PURCHASE_RECEIPT_SEED), buyer.toBuffer(), game.toBuffer()],
    program(programId)
  );
}

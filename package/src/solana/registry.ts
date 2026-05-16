import anchorPkg, {
  type AnchorProvider,
  type Idl,
  type Program as AnchorProgram
} from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  type Signer,
  type TransactionInstruction
} from "@solana/web3.js";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import { getPgl1Idl, getRegistryIdl } from "./deployments";
import type { PeridotEnvironment } from "../core";
import {
  findAuthorizedRegistryProgramPda,
  findAuthorizedSourceProgramPda,
  findCreatorStatePda,
  findGamePaymentOptionPda,
  findGamePda,
  findGameStoreConfigPda,
  findPglConfigPda,
  findPublishGrantPda,
  findRegistryConfigPda,
  findRegistryGamePda,
  findRegistryPaymentTokenPda,
  findStoreAcceptedPaymentTokenPda
} from "./pda";
import { getSolanaProgramId } from "./programs";
import type { Pgl1 } from "./types/pgl1";
import type { Registry } from "./types/registry";

const { BN, Program } = anchorPkg;

export type RegistryGameStatus = "active" | "suspended" | "banned";

export type RegistryClientConfig = {
  provider: AnchorProvider;
  environment: PeridotEnvironment;
};

export type AnchorBn = InstanceType<typeof BN>;

export type NumberLike = AnchorBn | bigint | number | string;

export type RegistryGameRow = {
  publicKey: PublicKey;
  game: PublicKey;
  gameId: string;
  registeredAt: AnchorBn;
  status: RegistryGameStatus | "unknown";
};

export const REGISTRY_GAME_STATUS = {
  active: 0,
  suspended: 1,
  banned: 2
} as const satisfies Record<RegistryGameStatus, number>;

function toBn(value: NumberLike): AnchorBn {
  if (BN.isBN(value)) {
    return value;
  }

  return new BN(value.toString());
}

function statusToCode(status: RegistryGameStatus | number): number {
  if (typeof status === "number") {
    return status;
  }

  return REGISTRY_GAME_STATUS[status];
}

function statusFromAccount(status: unknown): RegistryGameRow["status"] {
  if (status && typeof status === "object") {
    if ("active" in status) return "active";
    if ("suspended" in status) return "suspended";
    if ("banned" in status) return "banned";
  }

  return "unknown";
}

export function createRegistryProgram(
  config: RegistryClientConfig
): AnchorProgram<Registry> {
  const programId = getSolanaProgramId({
    environment: config.environment,
    program: "registry"
  });

  const registryIdl = getRegistryIdl(config.environment);

  return new Program<Registry>(
    {
      ...registryIdl,
      address: programId.toBase58()
    } as unknown as Idl,
    config.provider
  );
}

export function createPgl1Program(
  config: RegistryClientConfig
): AnchorProgram<Pgl1> {
  const programId = getSolanaProgramId({
    environment: config.environment,
    program: "pgl1"
  });

  const pgl1Idl = getPgl1Idl(config.environment);

  return new Program<Pgl1>(
    {
      ...pgl1Idl,
      address: programId.toBase58()
    } as unknown as Idl,
    config.provider
  );
}

export function createRegistryClient(config: RegistryClientConfig) {
  const registryProgram = createRegistryProgram(config);
  const pgl1Program = createPgl1Program(config);
  const authority = config.provider.publicKey;
  const registryProgramId = registryProgram.programId;
  const pgl1ProgramId = getSolanaProgramId({
    environment: config.environment,
    program: "pgl1"
  });
  const storeProgramId = getSolanaProgramId({
    environment: config.environment,
    program: "store"
  });

  async function initializeRegistryInstruction(params: {
    treasury: PublicKey;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .initializeRegistry(params.treasury)
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        systemProgram: SystemProgram.programId
      })
      .instruction();
  }

  async function initializeRegistry(params: {
    treasury: PublicKey;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .initializeRegistry(params.treasury)
      .accounts({
        authority,
        config: configPda,
        systemProgram: SystemProgram.programId
      })
      .rpc();
  }

  async function setTreasuryInstruction(params: {
    treasury: PublicKey;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .setTreasury(params.treasury)
      .accounts({
        authority: params.authority ?? authority,
        config: configPda
      })
      .instruction();
  }

  async function setTreasury(params: { treasury: PublicKey }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .setTreasury(params.treasury)
      .accounts({
        authority,
        config: configPda
      })
      .rpc();
  }

  async function addPaymentTokenInstruction(params: {
    mint: PublicKey;
    feeAmount: NumberLike;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .addPaymentToken(toBn(params.feeAmount))
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken,
        systemProgram: SystemProgram.programId
      })
      .instruction();
  }

  async function addPaymentToken(params: {
    mint: PublicKey;
    feeAmount: NumberLike;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .addPaymentToken(toBn(params.feeAmount))
      .accounts({
        authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken,
        systemProgram: SystemProgram.programId
      })
      .rpc();
  }

  async function updatePaymentTokenInstruction(params: {
    mint: PublicKey;
    active: boolean;
    feeAmount: NumberLike;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .updatePaymentToken(params.active, toBn(params.feeAmount))
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken
      })
      .instruction();
  }

  async function updatePaymentToken(params: {
    mint: PublicKey;
    active: boolean;
    feeAmount: NumberLike;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .updatePaymentToken(params.active, toBn(params.feeAmount))
      .accounts({
        authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken
      })
      .rpc();
  }

  async function removePaymentTokenInstruction(params: {
    mint: PublicKey;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .removePaymentToken()
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken
      })
      .instruction();
  }

  async function removePaymentToken(params: {
    mint: PublicKey;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.mint,
      registryProgramId
    );

    return (registryProgram.methods as any)
      .removePaymentToken()
      .accounts({
        authority,
        config: configPda,
        mint: params.mint,
        acceptedPaymentToken
      })
      .rpc();
  }

  async function createPublishGrantInstruction(params: {
    publisher?: PublicKey;
    expiredAt?: NumberLike | null;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const publisher = params.publisher ?? authority;
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);

    return (registryProgram.methods as any)
      .createPublishGrant(
        params.expiredAt === undefined || params.expiredAt === null
          ? null
          : toBn(params.expiredAt)
      )
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        publisher,
        publishGrant,
        systemProgram: SystemProgram.programId
      })
      .instruction();
  }

  async function createPublishGrant(params: {
    publisher?: PublicKey;
    expiredAt?: NumberLike | null;
    signers?: Signer[];
  }): Promise<string> {
    const publisher = params.publisher ?? authority;
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);

    return (registryProgram.methods as any)
      .createPublishGrant(
        params.expiredAt === undefined || params.expiredAt === null
          ? null
          : toBn(params.expiredAt)
      )
      .accounts({
        authority,
        config: configPda,
        publisher,
        publishGrant,
        systemProgram: SystemProgram.programId
      })
      .signers(params.signers ?? [])
      .rpc();
  }

  async function updatePublishGrantInstruction(params: {
    publisher?: PublicKey;
    expiredAt?: NumberLike | null;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const publisher = params.publisher ?? authority;
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);

    return (registryProgram.methods as any)
      .updatePublishGrant(
        params.expiredAt === undefined || params.expiredAt === null
          ? null
          : toBn(params.expiredAt)
      )
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        publisher,
        publishGrant
      })
      .instruction();
  }

  async function updatePublishGrant(params: {
    publisher?: PublicKey;
    expiredAt?: NumberLike | null;
    signers?: Signer[];
  }): Promise<string> {
    const publisher = params.publisher ?? authority;
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);

    return (registryProgram.methods as any)
      .updatePublishGrant(
        params.expiredAt === undefined || params.expiredAt === null
          ? null
          : toBn(params.expiredAt)
      )
      .accounts({
        authority,
        config: configPda,
        publisher,
        publishGrant
      })
      .signers(params.signers ?? [])
      .rpc();
  }

  async function resolveCreateGameAndRegisterAccounts(params: {
    publisher?: PublicKey;
    paymentMint: PublicKey;
    basePrice?: NumberLike | null;
    mintToken?: PublicKey | null;
    includePublishGrant?: boolean;
  }) {
    const publisher = params.publisher ?? authority;
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const registryConfig = await registryProgram.account.registryConfig.fetch(
      configPda
    );
    const [acceptedPaymentToken] = findRegistryPaymentTokenPda(
      params.paymentMint,
      registryProgramId
    );
    const [pglCreatorState] = findCreatorStatePda(publisher, pgl1ProgramId);

    let nextNonce = 0n;
    try {
      const creatorState = await pgl1Program.account.creatorState.fetch(
        pglCreatorState
      );
      nextNonce = BigInt(creatorState.nextNonce.toString());
    } catch {
      nextNonce = 0n;
    }

    const [game] = findGamePda(publisher, nextNonce, pgl1ProgramId);
    const [registryGame] = findRegistryGamePda(game, registryProgramId);
    const [pglConfig] = findPglConfigPda(pgl1ProgramId);
    const pglConfigAccount = await pgl1Program.account.pglConfig.fetch(
      pglConfig
    );
    const [storeAuthorizedSourceProgram] = findAuthorizedSourceProgramPda(
      pgl1ProgramId,
      storeProgramId
    );
    const [storeAuthorizedRegistryProgram] = findAuthorizedRegistryProgramPda(
      registryProgramId,
      storeProgramId
    );
    const [storeGameStoreConfig] = findGameStoreConfigPda(game, storeProgramId);
    const publisherPaymentAccount = getAssociatedTokenAddressSync(
      params.paymentMint,
      publisher
    );
    const treasuryPaymentAccount = getAssociatedTokenAddressSync(
      params.paymentMint,
      registryConfig.treasury
    );
    const remainingAccounts: {
      pubkey: PublicKey;
      isWritable: boolean;
      isSigner: boolean;
    }[] = [];

    if (params.includePublishGrant) {
      const [publishGrant] = findPublishGrantPda(publisher, registryProgramId);
      remainingAccounts.push({
        pubkey: publishGrant,
        isWritable: false,
        isSigner: false
      });
    }

    if (params.basePrice && params.mintToken) {
      const [storeAcceptedPaymentToken] = findStoreAcceptedPaymentTokenPda(
        params.mintToken,
        storeProgramId
      );
      const [gamePaymentOption] = findGamePaymentOptionPda(
        game,
        params.mintToken,
        storeProgramId
      );

      remainingAccounts.push(
        { pubkey: storeAcceptedPaymentToken, isWritable: false, isSigner: false },
        { pubkey: gamePaymentOption, isWritable: true, isSigner: false }
      );
    }

    return {
      accounts: {
        publisher,
        config: configPda,
        paymentMint: params.paymentMint,
        acceptedPaymentToken,
        publisherPaymentAccount,
        treasuryPaymentAccount,
        registryGame,
        game,
        pglCreatorState,
        pglConfig,
        pglTreasury: pglConfigAccount.treasury,
        pgl1Program: pgl1ProgramId,
        storeProgram: storeProgramId,
        storeAuthorizedSourceProgram,
        storeAuthorizedRegistryProgram,
        storeGameStoreConfig,
        registryProgram: registryProgramId,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      },
      remainingAccounts
    };
  }

  async function createGameAndRegisterInstruction(params: {
    gameId: string;
    metadataUri: string;
    paymentMint: PublicKey;
    basePrice?: NumberLike | null;
    mintToken?: PublicKey | null;
    includePublishGrant?: boolean;
    publisher?: PublicKey;
  }): Promise<TransactionInstruction> {
    const resolved = await resolveCreateGameAndRegisterAccounts(params);

    return (registryProgram.methods as any)
      .createGameAndRegister(
        params.gameId,
        params.metadataUri,
        params.basePrice === undefined || params.basePrice === null
          ? null
          : toBn(params.basePrice),
        params.mintToken ?? null
      )
      .accounts(resolved.accounts)
      .remainingAccounts(resolved.remainingAccounts)
      .instruction();
  }

  async function createGameAndRegister(params: {
    gameId: string;
    metadataUri: string;
    paymentMint: PublicKey;
    basePrice?: NumberLike | null;
    mintToken?: PublicKey | null;
    includePublishGrant?: boolean;
    signers?: Signer[];
  }): Promise<string> {
    const resolved = await resolveCreateGameAndRegisterAccounts(params);

    return (registryProgram.methods as any)
      .createGameAndRegister(
        params.gameId,
        params.metadataUri,
        params.basePrice === undefined || params.basePrice === null
          ? null
          : toBn(params.basePrice),
        params.mintToken ?? null
      )
      .accounts(resolved.accounts)
      .remainingAccounts(resolved.remainingAccounts)
      .signers(params.signers ?? [])
      .rpc();
  }

  async function updateGameStatusInstruction(params: {
    registryGame: PublicKey;
    status: RegistryGameStatus | number;
    authority?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .updateGameStatus(statusToCode(params.status))
      .accounts({
        authority: params.authority ?? authority,
        config: configPda,
        registryGame: params.registryGame
      })
      .instruction();
  }

  async function updateGameStatus(params: {
    registryGame: PublicKey;
    status: RegistryGameStatus | number;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);

    return (registryProgram.methods as any)
      .updateGameStatus(statusToCode(params.status))
      .accounts({
        authority,
        config: configPda,
        registryGame: params.registryGame
      })
      .rpc();
  }

  async function closeRegistryGameInstruction(params: {
    game: PublicKey;
    registryGame: PublicKey;
    publisher?: PublicKey;
  }): Promise<TransactionInstruction> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const registryConfig = await registryProgram.account.registryConfig.fetch(
      configPda
    );

    return (registryProgram.methods as any)
      .closeRegistryGame()
      .accounts({
        publisher: params.publisher ?? authority,
        config: configPda,
        game: params.game,
        registryGame: params.registryGame,
        treasury: registryConfig.treasury
      })
      .instruction();
  }

  async function closeRegistryGame(params: {
    game: PublicKey;
    registryGame: PublicKey;
  }): Promise<string> {
    const [configPda] = findRegistryConfigPda(registryProgramId);
    const registryConfig = await registryProgram.account.registryConfig.fetch(
      configPda
    );

    return (registryProgram.methods as any)
      .closeRegistryGame()
      .accounts({
        publisher: authority,
        config: configPda,
        game: params.game,
        registryGame: params.registryGame,
        treasury: registryConfig.treasury
      })
      .rpc();
  }

  async function fetchGames(): Promise<RegistryGameRow[]> {
    const games = await registryProgram.account.registryGame.all();

    return games.map(({ publicKey, account }) => ({
      publicKey,
      game: account.game,
      gameId: account.gameId,
      registeredAt: account.registeredAt,
      status: statusFromAccount(account.status)
    }));
  }

  async function fetchGame(registryGame: PublicKey) {
    return registryProgram.account.registryGame.fetch(registryGame);
  }

  return {
    provider: config.provider,
    environment: config.environment,
    registryProgram,
    pgl1Program,
    programId: registryProgramId,
    initializeRegistryInstruction,
    initializeRegistry,
    setTreasuryInstruction,
    setTreasury,
    addPaymentTokenInstruction,
    addPaymentToken,
    updatePaymentTokenInstruction,
    updatePaymentToken,
    removePaymentTokenInstruction,
    removePaymentToken,
    createPublishGrantInstruction,
    createPublishGrant,
    updatePublishGrantInstruction,
    updatePublishGrant,
    createGameAndRegisterInstruction,
    createGameAndRegister,
    updateGameStatusInstruction,
    updateGameStatus,
    closeRegistryGameInstruction,
    closeRegistryGame,
    fetchGames,
    fetchGame
  };
}

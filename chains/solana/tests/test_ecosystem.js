const anchor = require("@coral-xyz/anchor");
const { Keypair, PublicKey, SystemProgram, Transaction } = require("@solana/web3.js");
const { createMint, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } = require("@solana/spl-token");

const pglIdl = require("../target/idl/pgl1.json");
const registryIdl = require("../target/idl/registry.json");

function derivePda(seeds, programId) {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

function u64LeBuffer(value) {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(value));
  return b;
}

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet.payer;
  const pglProgram = new anchor.Program(pglIdl, provider);
  const registryProgram = new anchor.Program(registryIdl, provider);

  const pglConfigPda = derivePda([Buffer.from("pgl_config")], pglProgram.programId);
  const registryConfigPda = derivePda([Buffer.from("registry_config")], registryProgram.programId);

  if (!(await provider.connection.getAccountInfo(pglConfigPda))) {
    await pglProgram.methods
      .initializePgl(wallet.publicKey, new anchor.BN(0))
      .accounts({ authority: wallet.publicKey, pglConfig: pglConfigPda, systemProgram: SystemProgram.programId })
      .rpc();
  }

  if (!(await provider.connection.getAccountInfo(registryConfigPda))) {
    await registryProgram.methods
      .initializeRegistry(wallet.publicKey)
      .accounts({ authority: wallet.publicKey, config: registryConfigPda, systemProgram: SystemProgram.programId })
      .rpc();
  }

  const mint = await createMint(provider.connection, wallet, wallet.publicKey, null, 6);
  const acceptedTokenPda = derivePda([Buffer.from("accepted_payment_token"), mint.toBuffer()], registryProgram.programId);
  if (!(await provider.connection.getAccountInfo(acceptedTokenPda))) {
    await registryProgram.methods
      .addPaymentToken(new anchor.BN(1000))
      .accounts({
        authority: wallet.publicKey,
        config: registryConfigPda,
        mint,
        acceptedPaymentToken: acceptedTokenPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  const publisher = Keypair.generate();
  await provider.sendAndConfirm(
    new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: provider.publicKey,
        toPubkey: publisher.publicKey,
        lamports: 2 * anchor.web3.LAMPORTS_PER_SOL,
      }),
    ),
  );

  const publisherPayment = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet,
    mint,
    publisher.publicKey,
  );
  const treasuryPayment = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    wallet,
    mint,
    wallet.publicKey,
  );

  const publishGrantPda = derivePda(
    [Buffer.from("publish_grant"), publisher.publicKey.toBuffer()],
    registryProgram.programId,
  );
  await registryProgram.methods
    .setPublishGrant(null)
    .accounts({
      authority: wallet.publicKey,
      config: registryConfigPda,
      publisher: publisher.publicKey,
      publishGrant: publishGrantPda,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  const creatorStatePda = derivePda([Buffer.from("creator_state"), publisher.publicKey.toBuffer()], pglProgram.programId);
  let nextNonce = 0;
  if (await provider.connection.getAccountInfo(creatorStatePda)) {
    const creatorState = await pglProgram.account.creatorState.fetch(creatorStatePda);
    nextNonce = Number(creatorState.nextNonce.toString());
  }

  const gamePda = derivePda(
    [Buffer.from("game"), publisher.publicKey.toBuffer(), u64LeBuffer(nextNonce)],
    pglProgram.programId,
  );
  const registryGamePda = derivePda([Buffer.from("registry_game"), gamePda.toBuffer()], registryProgram.programId);

  await registryProgram.methods
    .createGameAndRegister(`smoke-${Date.now()}`, "https://meta.peridot/smoke.json")
    .accounts({
      publisher: publisher.publicKey,
      config: registryConfigPda,
      paymentMint: mint,
      acceptedPaymentToken: acceptedTokenPda,
      publisherPaymentAccount: publisherPayment.address,
      treasuryPaymentAccount: treasuryPayment.address,
      registryGame: registryGamePda,
      game: gamePda,
      pglCreatorState: creatorStatePda,
      pglConfig: pglConfigPda,
      pglTreasury: wallet.publicKey,
      pgl1Program: pglProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .remainingAccounts([{ pubkey: publishGrantPda, isSigner: false, isWritable: false }])
    .signers([publisher])
    .rpc();

  const registryGame = await registryProgram.account.registryGame.fetch(registryGamePda);
  console.log("Ecosystem smoke success:", {
    game: gamePda.toBase58(),
    registryGame: registryGamePda.toBase58(),
    status: registryGame.status,
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

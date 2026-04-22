import { expect } from "chai";
import { SystemProgram } from "@solana/web3.js";
import {
  createRegisteredGame,
  derivePda,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("gamer library", () => {
  it("lists all licenses owned by gamer", async () => {
    const base = await setupPeridotFixture();

    const authorizedActorPda = derivePda(
      [Buffer.from("authorized_actor"), base.authority.publicKey.toBuffer()],
      base.pglProgram.programId,
    );

    try {
      await base.pglProgram.account.authorizedActor.fetch(authorizedActorPda);
    } catch {
      await base.pglProgram.methods
        .addAuthorizedActor()
        .accounts({
          authority: base.authority.publicKey,
          actor: base.authority.publicKey,
          pglConfig: base.pglConfigPda,
          authorizedActor: authorizedActorPda,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    }

    const gameA = await createRegisteredGame(base);
    const licenseA = derivePda(
      [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gameA.gamePda.toBuffer()],
      base.pglProgram.programId,
    );

    await base.pglProgram.methods
      .mintLicense(null)
      .accounts({
        actor: base.authority.publicKey,
        holder: base.gamer.publicKey,
        authorizedActor: authorizedActorPda,
        game: gameA.gamePda,
        license: licenseA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const gameB = await createRegisteredGame(base);
    const licenseB = derivePda(
      [Buffer.from("license"), base.gamer.publicKey.toBuffer(), gameB.gamePda.toBuffer()],
      base.pglProgram.programId,
    );

    await base.pglProgram.methods
      .mintLicense(null)
      .accounts({
        actor: base.authority.publicKey,
        holder: base.gamer.publicKey,
        authorizedActor: authorizedActorPda,
        game: gameB.gamePda,
        license: licenseB,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const licenses = await base.pglProgram.account.license.all([
      {
        memcmp: {
          offset: 8,
          bytes: base.gamer.publicKey.toBase58(),
        },
      },
    ]);

    expect(licenses.length).to.be.greaterThan(1);
    const ownedGames = licenses.map((l: any) => l.account.game.toBase58());
    expect(ownedGames).to.include(gameA.gamePda.toBase58());
    expect(ownedGames).to.include(gameB.gamePda.toBase58());
  });
});

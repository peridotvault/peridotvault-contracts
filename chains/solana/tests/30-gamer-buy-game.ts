import { expect } from "chai";
import { SystemProgram } from "@solana/web3.js";
import {
  createRegisteredGame,
  derivePda,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("gamer license flow", () => {
  it("mints a license for gamer via authorized actor", async () => {
    const base = await setupPeridotFixture();
    const game = await createRegisteredGame(base);

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

    const licensePda = derivePda(
      [Buffer.from("license"), base.gamer.publicKey.toBuffer(), game.gamePda.toBuffer()],
      base.pglProgram.programId,
    );

    await base.pglProgram.methods
      .mintLicense(null)
      .accounts({
        actor: base.authority.publicKey,
        holder: base.gamer.publicKey,
        authorizedActor: authorizedActorPda,
        game: game.gamePda,
        license: licensePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const license = (await base.pglProgram.account.license.fetch(licensePda)) as any;
    expect(license.holder.toBase58()).to.eq(base.gamer.publicKey.toBase58());
    expect(license.game.toBase58()).to.eq(game.gamePda.toBase58());
  });
});

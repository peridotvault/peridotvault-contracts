import { expect } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  setupPeridotFixture,
  derivePda,
} from "./helpers/peridot";

// ANT-941 (FR4-ON): pgl1 `revoke_license` — burn/close path.
// Covers: mint -> revoke closes PDA + LicenseRevoked event + rent refund,
// unauthorized actor cannot revoke, double revoke fails closed.
async function deriveNextGamePda(
  base: Awaited<ReturnType<typeof setupPeridotFixture>>,
  publisher: Keypair,
): Promise<PublicKey> {
  const creatorStatePda = derivePda(
    [Buffer.from("creator_state"), publisher.publicKey.toBuffer()],
    base.pglProgram.programId,
  );
  let nextNonce = BigInt(0);
  if (await base.provider.connection.getAccountInfo(creatorStatePda)) {
    const creatorState = (await base.pglProgram.account.creatorState.fetch(
      creatorStatePda,
    )) as any;
    nextNonce = BigInt(creatorState.nextNonce.toString());
  }
  const nonceBuf = Buffer.alloc(8);
  nonceBuf.writeBigUInt64LE(nextNonce);
  return derivePda(
    [Buffer.from("game"), publisher.publicKey.toBuffer(), nonceBuf],
    base.pglProgram.programId,
  );
}

async function mintLicenseFor(
  base: Awaited<ReturnType<typeof setupPeridotFixture>>,
  tag: string,
) {
  const gameId = `revoke-game-${tag}-${Date.now()}`;
  const metadataUri = "https://example.com/game.json";
  const creatorStatePda = derivePda(
    [Buffer.from("creator_state"), base.publisher.publicKey.toBuffer()],
    base.pglProgram.programId,
  );
  const gamePda = await deriveNextGamePda(base, base.publisher);
  await base.pglProgram.methods
    .createGame(gameId, metadataUri)
    .accounts({
      creator: base.publisher.publicKey,
      pglConfig: base.pglConfigPda,
      treasury: base.authority.publicKey,
      creatorState: creatorStatePda,
      game: gamePda,
      systemProgram: SystemProgram.programId,
    })
    .signers([base.publisher])
    .rpc();

  const licensePda = derivePda(
    [
      Buffer.from("license"),
      base.gamer.publicKey.toBuffer(),
      gamePda.toBuffer(),
    ],
    base.pglProgram.programId,
  );
  await base.pglProgram.methods
    .mintLicense(null)
    .accounts({
      actor: base.authority.publicKey,
      holder: base.gamer.publicKey,
      authorizedActor: base.storeActorAuthorizedPda,
      game: gamePda,
      license: licensePda,
      systemProgram: SystemProgram.programId,
    })
    .signers([base.authority, base.gamer])
    .rpc();
  return { gamePda, licensePda };
}

describe("pgl1 revoke_license (ANT-941 FR4-ON)", () => {
  it("revokes: closes license PDA, emits LicenseRevoked, refunds rent", async () => {
    const base = await setupPeridotFixture();
    const { gamePda, licensePda } = await mintLicenseFor(base, "basic");

    const before = await base.provider.connection.getBalance(
      base.gamer.publicKey,
    );

    const revoked = new Promise<any>((resolve) => {
      const sub = (base.pglProgram as any).addEventListener(
        "licenseRevoked",
        (event: any) => {
          (base.pglProgram as any).removeEventListener(sub).then(() =>
            resolve(event),
          );
        },
      );
    });

    await base.pglProgram.methods
      .revokeLicense()
      .accounts({
        actor: base.authority.publicKey,
        holder: base.gamer.publicKey,
        authorizedActor: base.storeActorAuthorizedPda,
        game: gamePda,
        license: licensePda,
      })
      .signers([base.authority])
      .rpc();

    const event = (await revoked) as any;
    expect(event.holder.toBase58()).to.eq(base.gamer.publicKey.toBase58());
    expect(event.game.toBase58()).to.eq(gamePda.toBase58());
    expect(event.actor.toBase58()).to.eq(base.authority.publicKey.toBase58());

    // PDA closed: fetch must fail.
    let closed = false;
    try {
      await base.pglProgram.account.license.fetch(licensePda);
    } catch {
      closed = true;
    }
    expect(closed).to.eq(true);

    // Rent refunded to holder (balance grew vs pre-revoke).
    const after = await base.provider.connection.getBalance(
      base.gamer.publicKey,
    );
    expect(after).to.be.greaterThan(before);
  });

  it("rejects revoke from non-authorized actor (access control)", async () => {
    const base = await setupPeridotFixture();
    const { gamePda, licensePda } = await mintLicenseFor(base, "acl");
    const stranger = Keypair.generate();

    let failed = false;
    try {
      await base.pglProgram.methods
        .revokeLicense()
        .accounts({
          actor: stranger.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
        })
        .signers([stranger])
        .rpc();
    } catch {
      failed = true;
    }
    expect(failed).to.eq(true);

    // License untouched.
    const license = (await base.pglProgram.account.license.fetch(
      licensePda,
    )) as any;
    expect(license.holder.toBase58()).to.eq(base.gamer.publicKey.toBase58());
  });

  it("double revoke fails closed (license gone)", async () => {
    const base = await setupPeridotFixture();
    const { gamePda, licensePda } = await mintLicenseFor(base, "double");

    await base.pglProgram.methods
      .revokeLicense()
      .accounts({
        actor: base.authority.publicKey,
        holder: base.gamer.publicKey,
        authorizedActor: base.storeActorAuthorizedPda,
        game: gamePda,
        license: licensePda,
      })
      .signers([base.authority])
      .rpc();

    let failed = false;
    try {
      await base.pglProgram.methods
        .revokeLicense()
        .accounts({
          actor: base.authority.publicKey,
          holder: base.gamer.publicKey,
          authorizedActor: base.storeActorAuthorizedPda,
          game: gamePda,
          license: licensePda,
        })
        .signers([base.authority])
        .rpc();
    } catch {
      failed = true;
    }
    expect(failed).to.eq(true);
  });
});

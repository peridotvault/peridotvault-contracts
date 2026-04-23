import { expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import {
  DEFAULT_MAX_REFERRAL_BPS,
  DEFAULT_REFERRAL_BPS,
  DEFAULT_PLATFORM_FEE_BPS,
  setupPeridotFixture,
} from "./helpers/peridot";

describe("admin config", () => {
  it("updates core admin configs across pgl1, registry, and store", async () => {
    const base = await setupPeridotFixture();

    const newTreasury = Keypair.generate().publicKey;
    const newCreateFee = new anchor.BN(12345);
    const newPlatformFee = 900;
    const newDefaultReferral = 300;
    const newMaxReferral = 4000;

    const pglBefore = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
    const registryBefore = (await base.registryProgram.account.registryConfig.fetch(
      base.registryConfigPda,
    )) as any;
    const storeBefore = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;

    await base.pglProgram.methods
      .setCreateGameFee(newCreateFee)
      .accounts({ authority: base.authority.publicKey, pglConfig: base.pglConfigPda })
      .rpc();

    await base.pglProgram.methods
      .setTreasury(newTreasury)
      .accounts({ authority: base.authority.publicKey, pglConfig: base.pglConfigPda })
      .rpc();

    await base.registryProgram.methods
      .setTreasury(newTreasury)
      .accounts({ authority: base.authority.publicKey, config: base.registryConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setPlatformFee(newPlatformFee)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setMaxReferral(newMaxReferral)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setDefaultReferral(newDefaultReferral)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setTreasury(newTreasury)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    const pglAfter = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
    const registryAfter = (await base.registryProgram.account.registryConfig.fetch(
      base.registryConfigPda,
    )) as any;
    const storeAfter = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;

    expect(pglAfter.treasury.toBase58()).to.eq(newTreasury.toBase58());
    expect(pglAfter.createGameFeeLamports.toString()).to.eq(newCreateFee.toString());
    expect(registryAfter.treasury.toBase58()).to.eq(newTreasury.toBase58());
    expect(registryAfter.pgl1Program.toBase58()).to.eq(base.pglProgram.programId.toBase58());
    expect(storeAfter.treasury.toBase58()).to.eq(newTreasury.toBase58());
    expect(storeAfter.platformFeeBps).to.eq(newPlatformFee);
    expect(storeAfter.defaultReferralBps).to.eq(newDefaultReferral);
    expect(storeAfter.maxReferralBps).to.eq(newMaxReferral);

    await base.pglProgram.methods
      .setCreateGameFee(pglBefore.createGameFeeLamports)
      .accounts({ authority: base.authority.publicKey, pglConfig: base.pglConfigPda })
      .rpc();

    await base.pglProgram.methods
      .setTreasury(pglBefore.treasury)
      .accounts({ authority: base.authority.publicKey, pglConfig: base.pglConfigPda })
      .rpc();

    await base.registryProgram.methods
      .setTreasury(registryBefore.treasury)
      .accounts({ authority: base.authority.publicKey, config: base.registryConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setPlatformFee(DEFAULT_PLATFORM_FEE_BPS)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setMaxReferral(DEFAULT_MAX_REFERRAL_BPS)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setDefaultReferral(DEFAULT_REFERRAL_BPS)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();

    await base.storeProgram.methods
      .setTreasury(storeBefore.treasury)
      .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
      .rpc();
  });

  it("rejects default treasury for store", async () => {
    const base = await setupPeridotFixture();

    let failed = false;
    try {
      await base.storeProgram.methods
        .setTreasury(PublicKey.default)
        .accounts({ authority: base.authority.publicKey, storeConfig: base.storeConfigPda })
        .rpc();
    } catch (error: any) {
      failed = true;
      expect(String(error)).to.include("Invalid treasury");
    }

    expect(failed).to.eq(true);
  });
});

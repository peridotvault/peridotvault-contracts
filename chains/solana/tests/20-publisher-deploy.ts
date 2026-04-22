import { expect } from "chai";
import { setupPeridotFixture } from "./helpers/peridot";

describe("publisher deploy flow", () => {
  it("has initialized configs and core allowlists", async () => {
    const base = await setupPeridotFixture();

    const pglConfig = (await base.pglProgram.account.pglConfig.fetch(base.pglConfigPda)) as any;
    const registryConfig = (await base.registryProgram.account.registryConfig.fetch(
      base.registryConfigPda,
    )) as any;
    const storeConfig = (await base.storeProgram.account.storeConfig.fetch(base.storeConfigPda)) as any;

    expect(pglConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
    expect(registryConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());
    expect(storeConfig.authority.toBase58()).to.eq(base.authority.publicKey.toBase58());

    const registryToken = (await base.registryProgram.account.acceptedPaymentToken.fetch(
      base.registryAcceptedPaymentTokenPda,
    )) as any;
    const storeToken = (await base.storeProgram.account.acceptedPaymentToken.fetch(
      base.storeAcceptedPaymentTokenPda,
    )) as any;

    expect(registryToken.active).to.eq(true);
    expect(storeToken.active).to.eq(true);
  });
});

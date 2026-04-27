use crate::{
    events::PaymentTokenRemoved,
    state::{
        AcceptedPaymentToken, RegistryConfig, ACCEPTED_PAYMENT_TOKEN_SEED, REGISTRY_CONFIG_SEED,
    },
};
use quasar_lang::prelude::*;
use quasar_spl::{InterfaceAccount, Mint};
#[derive(Accounts)]
pub struct RemovePaymentToken<'info> {
    pub authority: &'info mut Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    pub mint: &'info InterfaceAccount<Mint>,
    #[account(mut, seeds=[ACCEPTED_PAYMENT_TOKEN_SEED, mint], bump=accepted_payment_token.bump)]
    pub accepted_payment_token: &'info mut Account<AcceptedPaymentToken>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, RemovePaymentToken<'info>>,
) -> Result<(), ProgramError> {
    emit!(PaymentTokenRemoved {
        mint: *ctx.accounts.mint.address()
    })?;
    ctx.accounts
        .accepted_payment_token
        .close(ctx.accounts.authority.to_account_view())?;
    Ok(())
}

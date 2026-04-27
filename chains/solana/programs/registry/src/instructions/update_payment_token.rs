use crate::{
    errors::RegistryError,
    events::PaymentTokenUpdated,
    state::{
        AcceptedPaymentToken, RegistryConfig, ACCEPTED_PAYMENT_TOKEN_SEED, REGISTRY_CONFIG_SEED,
    },
};
use quasar_lang::prelude::*;
use quasar_spl::{InterfaceAccount, Mint};
#[derive(Accounts)]
pub struct UpdatePaymentToken<'info> {
    pub authority: &'info Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    pub mint: &'info InterfaceAccount<Mint>,
    #[account(mut, seeds=[ACCEPTED_PAYMENT_TOKEN_SEED, mint], bump=accepted_payment_token.bump)]
    pub accepted_payment_token: &'info mut Account<AcceptedPaymentToken>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, UpdatePaymentToken<'info>>,
    active: bool,
    fee_amount: u64,
) -> Result<(), ProgramError> {
    require!(fee_amount > 0, RegistryError::InvalidFeeAmount);
    let old_active = ctx.accounts.accepted_payment_token.active.get();
    let old_fee_amount = ctx.accounts.accepted_payment_token.fee_amount.get();
    ctx.accounts.accepted_payment_token.active = active.into();
    ctx.accounts.accepted_payment_token.fee_amount = fee_amount.into();
    emit!(PaymentTokenUpdated {
        mint: ctx.accounts.accepted_payment_token.mint,
        old_active,
        new_active: active,
        old_fee_amount,
        new_fee_amount: fee_amount
    })?;
    Ok(())
}

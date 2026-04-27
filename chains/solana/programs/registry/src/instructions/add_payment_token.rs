use crate::{
    errors::RegistryError,
    events::PaymentTokenAdded,
    state::{
        AcceptedPaymentToken, RegistryConfig, ACCEPTED_PAYMENT_TOKEN_SEED, REGISTRY_CONFIG_SEED,
    },
};
use quasar_lang::prelude::*;
use quasar_spl::{InterfaceAccount, Mint};
#[derive(Accounts)]
pub struct AddPaymentToken<'info> {
    pub authority: &'info mut Signer,
    #[account(seeds=[REGISTRY_CONFIG_SEED], bump=config.bump, has_one=authority)]
    pub config: &'info Account<RegistryConfig>,
    pub mint: &'info InterfaceAccount<Mint>,
    #[account(init, payer=authority, space=<AcceptedPaymentToken as Space>::SPACE, seeds=[ACCEPTED_PAYMENT_TOKEN_SEED, mint], bump)]
    pub accepted_payment_token: &'info mut Account<AcceptedPaymentToken>,
    pub system_program: &'info Program<System>,
}
pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, AddPaymentToken<'info>>,
    fee_amount: u64,
) -> Result<(), ProgramError> {
    require!(fee_amount > 0, RegistryError::InvalidFeeAmount);
    ctx.accounts.accepted_payment_token.set_inner(
        *ctx.accounts.mint.address(),
        true,
        fee_amount,
        ctx.bumps.accepted_payment_token,
    );
    emit!(PaymentTokenAdded {
        mint: *ctx.accounts.mint.address(),
        fee_amount
    })?;
    Ok(())
}

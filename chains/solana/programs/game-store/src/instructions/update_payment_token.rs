use quasar_lang::prelude::*;

use crate::{
    events::PaymentTokenUpdated,
    state::{AcceptedPaymentToken, StoreConfig},
};

#[derive(Accounts)]
pub struct UpdatePaymentToken<'info> {
    pub authority: &'info Signer,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info Account<StoreConfig>,
    #[account(mut)]
    pub accepted_payment_token: &'info mut Account<AcceptedPaymentToken>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, UpdatePaymentToken<'info>>,
    active: bool,
) -> Result<(), ProgramError> {
    ctx.accounts.accepted_payment_token.active = active.into();
    emit!(PaymentTokenUpdated {
        mint: ctx.accounts.accepted_payment_token.mint,
        active
    })?;
    Ok(())
}

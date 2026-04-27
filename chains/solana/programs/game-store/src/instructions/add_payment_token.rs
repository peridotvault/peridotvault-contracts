use quasar_lang::prelude::*;
use quasar_spl::{InterfaceAccount, Mint};

use crate::{
    events::PaymentTokenAdded,
    state::{AcceptedPaymentToken, StoreConfig},
};

#[derive(Accounts)]
pub struct AddPaymentToken<'info> {
    pub authority: &'info mut Signer,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info Account<StoreConfig>,
    pub mint: &'info InterfaceAccount<Mint>,
    #[account(
        init,
        payer = authority,
        space = <AcceptedPaymentToken as Space>::SPACE,
        seeds = [b"accepted_payment_token", mint],
        bump
    )]
    pub accepted_payment_token: &'info mut Account<AcceptedPaymentToken>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, AddPaymentToken<'info>>,
) -> Result<(), ProgramError> {
    ctx.accounts.accepted_payment_token.set_inner(
        *ctx.accounts.mint.address(),
        true,
        ctx.bumps.accepted_payment_token,
    );

    emit!(PaymentTokenAdded {
        mint: *ctx.accounts.mint.address()
    })?;
    Ok(())
}

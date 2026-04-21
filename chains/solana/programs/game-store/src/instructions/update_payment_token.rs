use anchor_lang::prelude::*;

use crate::{events::PaymentTokenUpdated, state::{AcceptedPaymentToken, StoreConfig}};

#[derive(Accounts)]
pub struct UpdatePaymentToken<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    #[account(
        mut,
        seeds = [b"accepted_payment_token", accepted_payment_token.mint.as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
}

pub fn handler(ctx: Context<UpdatePaymentToken>, active: bool) -> Result<()> {
    let token = &mut ctx.accounts.accepted_payment_token;
    token.active = active;
    emit!(PaymentTokenUpdated { mint: token.mint, active });
    Ok(())
}

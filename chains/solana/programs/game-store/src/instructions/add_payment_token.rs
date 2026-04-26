use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{events::PaymentTokenAdded, state::{AcceptedPaymentToken, StoreConfig}};

#[derive(Accounts)]
pub struct AddPaymentToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = authority,
        space = AcceptedPaymentToken::SPACE,
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<AddPaymentToken>) -> Result<()> {
    let token = &mut ctx.accounts.accepted_payment_token;
    token.mint = ctx.accounts.mint.key();
    token.active = true;
    token.bump = ctx.bumps.accepted_payment_token;

    emit!(PaymentTokenAdded { mint: token.mint });
    Ok(())
}

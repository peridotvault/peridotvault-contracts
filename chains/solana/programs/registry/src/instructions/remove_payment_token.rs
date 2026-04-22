use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    errors::RegistryError,
    events::PaymentTokenRemoved,
    state::{AcceptedPaymentToken, RegistryConfig},
};

#[derive(Accounts)]
pub struct RemovePaymentToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"registry_config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = authority,
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
}

pub(crate) fn handler(ctx: Context<RemovePaymentToken>) -> Result<()> {
    emit!(PaymentTokenRemoved {
        mint: ctx.accounts.mint.key(),
    });

    Ok(())
}

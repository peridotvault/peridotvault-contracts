use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    errors::RegistryError,
    events::PaymentTokenUpdated,
    state::{AcceptedPaymentToken, RegistryConfig},
};

#[derive(Accounts)]
pub struct UpdatePaymentToken<'info> {
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
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,
}

pub fn handler(ctx: Context<UpdatePaymentToken>, active: bool, fee_amount: u64) -> Result<()> {
    require!(fee_amount > 0, RegistryError::InvalidFeeAmount);

    let token = &mut ctx.accounts.accepted_payment_token;
    token.active = active;
    token.fee_amount = fee_amount;

    emit!(PaymentTokenUpdated {
        mint: token.mint,
        active,
        fee_amount,
    });

    Ok(())
}

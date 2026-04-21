use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{
    errors::RegistryError,
    events::PaymentTokenAdded,
    state::{AcceptedPaymentToken, RegistryConfig},
};

#[derive(Accounts)]
pub struct AddPaymentToken<'info> {
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
        init,
        payer = authority,
        space = AcceptedPaymentToken::SPACE,
        seeds = [b"accepted_payment_token", mint.key().as_ref()],
        bump
    )]
    pub accepted_payment_token: Account<'info, AcceptedPaymentToken>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddPaymentToken>, fee_amount: u64) -> Result<()> {
    require!(fee_amount > 0, RegistryError::InvalidFeeAmount);

    let token = &mut ctx.accounts.accepted_payment_token;
    token.mint = ctx.accounts.mint.key();
    token.active = true;
    token.fee_amount = fee_amount;
    token.bump = ctx.bumps.accepted_payment_token;

    emit!(PaymentTokenAdded {
        mint: token.mint,
        fee_amount,
    });

    Ok(())
}

use anchor_lang::prelude::*;

use crate::{
    constants::REGISTRY_STATE_SEED,
    errors::RegistryError,
    events::FeeExemptionUpdated,
    states::RegistryState,
};

#[derive(Accounts)]
pub struct SetFeeExemption<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump,
        has_one = governance @ RegistryError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
}

pub fn handler(ctx: Context<SetFeeExemption>, account: Pubkey, is_exempt: bool) -> Result<()> {
    require!(
        account != Pubkey::default(),
        RegistryError::InvalidFeeExemptionAccount
    );

    let registry_state = &mut ctx.accounts.registry_state;
    registry_state.set_fee_exemption(account, is_exempt)?;

    emit!(FeeExemptionUpdated { account, is_exempt });

    Ok(())
}

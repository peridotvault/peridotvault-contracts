use quasar_lang::prelude::*;

use crate::{errors::StoreError, events::TreasuryUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: &'info Signer,
    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: &'info mut Account<StoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetTreasury<'info>>,
    treasury: Address,
) -> Result<(), ProgramError> {
    require!(treasury != Address::default(), StoreError::InvalidTreasury);
    ctx.accounts.store_config.treasury = treasury;

    emit!(TreasuryUpdated { treasury })?;
    Ok(())
}

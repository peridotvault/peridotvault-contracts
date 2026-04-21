use anchor_lang::prelude::*;

use crate::{events::TreasuryUpdated, state::StoreConfig};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"store_config"],
        bump = store_config.bump,
        has_one = authority
    )]
    pub store_config: Account<'info, StoreConfig>,
}

pub fn handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    ctx.accounts.store_config.treasury = treasury;

    emit!(TreasuryUpdated { treasury });
    Ok(())
}

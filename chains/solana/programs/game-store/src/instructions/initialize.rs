use anchor_lang::prelude::*;
use crate::Initialize;

pub fn handler(
    ctx: Context<Initialize>,
    governance: Pubkey,
    treasury: Pubkey,
    platform_fee_bps: u16,
) -> Result<()> {
    let config = &mut ctx.accounts.store_config;
    config.governance = governance;
    config.treasury = treasury;
    config.platform_fee_bps = platform_fee_bps;
    config.bump = ctx.bumps.store_config;

    Ok(())
}

use anchor_lang::prelude::*;

use crate::{
    constants::{MAX_FEE_BPS, STORE_STATE_SEED},
    errors::GameStoreError,
    events::StoreInitialized,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(governance: Pubkey, treasury: Pubkey, registry: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = StoreState::SPACE,
        seeds = [STORE_STATE_SEED],
        bump
    )]
    pub store_state: Account<'info, StoreState>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Initialize>,
    governance: Pubkey,
    treasury: Pubkey,
    registry: Pubkey,
    platform_fee_bps: u16,
) -> Result<()> {
    require!(governance != Pubkey::default(), GameStoreError::InvalidGovernance);
    require!(treasury != Pubkey::default(), GameStoreError::InvalidTreasury);
    require!(registry != Pubkey::default(), GameStoreError::InvalidRegistry);
    require!(platform_fee_bps <= MAX_FEE_BPS, GameStoreError::InvalidPlatformFeeBps);

    let store_state = &mut ctx.accounts.store_state;
    store_state.bump = ctx.bumps.store_state;
    store_state.registry = registry;
    store_state.governance = governance;
    store_state.treasury = treasury;
    store_state.platform_fee_bps = platform_fee_bps;
    store_state.prices = Vec::new();
    store_state.publisher_balances = Vec::new();

    emit!(StoreInitialized {
        governance,
        treasury,
        registry,
        platform_fee_bps,
    });

    Ok(())
}

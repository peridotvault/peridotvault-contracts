use anchor_lang::prelude::*;

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    events::TreasuryUpdated,
    states::StoreState,
};

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub governance: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump,
        has_one = governance @ GameStoreError::Unauthorized
    )]
    pub store_state: Account<'info, StoreState>,
}

pub fn handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    require!(treasury != Pubkey::default(), GameStoreError::InvalidTreasury);

    let store_state = &mut ctx.accounts.store_state;
    let old_treasury = store_state.treasury;
    store_state.treasury = treasury;

    emit!(TreasuryUpdated {
        old_treasury,
        new_treasury: treasury,
    });

    Ok(())
}

use anchor_lang::prelude::*;

use crate::{
    constants::STORE_STATE_SEED,
    errors::GameStoreError,
    states::{PriceConfig, StoreState},
};

#[derive(Accounts)]
pub struct GetStoreView<'info> {
    #[account(
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,
}

pub fn get_price_config(ctx: Context<GetStoreView>, game_id: String) -> Result<PriceConfig> {
    ctx.accounts
        .store_state
        .price_config(&game_id)
        .cloned()
        .ok_or(error!(GameStoreError::PriceConfigNotFound))
}

pub fn get_publisher_balance(
    ctx: Context<GetStoreView>,
    publisher: Pubkey,
    token: Pubkey,
) -> Result<u64> {
    Ok(ctx.accounts.store_state.publisher_balance(&publisher, &token))
}

pub fn get_platform_fee(ctx: Context<GetStoreView>) -> Result<u16> {
    Ok(ctx.accounts.store_state.platform_fee_bps)
}

pub fn get_treasury(ctx: Context<GetStoreView>) -> Result<Pubkey> {
    Ok(ctx.accounts.store_state.treasury)
}

pub fn get_governance(ctx: Context<GetStoreView>) -> Result<Pubkey> {
    Ok(ctx.accounts.store_state.governance)
}

pub fn get_registry(ctx: Context<GetStoreView>) -> Result<Pubkey> {
    Ok(ctx.accounts.store_state.registry)
}

pub fn get_final_price(ctx: Context<GetStoreView>, game_id: String) -> Result<u64> {
    let price_config = ctx
        .accounts
        .store_state
        .price_config(&game_id)
        .ok_or(error!(GameStoreError::PriceConfigNotFound))?;
    Ok(StoreState::final_price(price_config))
}

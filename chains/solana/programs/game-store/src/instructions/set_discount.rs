use anchor_lang::prelude::*;
use pgc::states::GameState as PgcGameState;
use registry::states::{GameRegistration, RegistryState};

use crate::{
    constants::{MAX_FEE_BPS, MAX_GAME_ID_LEN, STORE_STATE_SEED},
    errors::GameStoreError,
    events::DiscountSet,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct SetDiscount<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [STORE_STATE_SEED],
        bump = store_state.bump
    )]
    pub store_state: Account<'info, StoreState>,

    #[account(address = store_state.registry)]
    pub registry_state: Account<'info, RegistryState>,

    pub pgc_game_state: Account<'info, PgcGameState>,
    pub game_registration: Account<'info, GameRegistration>,
}

pub fn handler(ctx: Context<SetDiscount>, game_id: String, discount_bps: u16) -> Result<()> {
    require!(!game_id.trim().is_empty(), GameStoreError::EmptyGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, GameStoreError::GameIdTooLong);
    require!(discount_bps <= MAX_FEE_BPS, GameStoreError::InvalidDiscountBps);

    let registry_game = &ctx.accounts.game_registration;
    require!(registry_game.game_id == game_id, GameStoreError::GameNotFound);

    require_keys_eq!(
        registry_game.contract_address,
        ctx.accounts.pgc_game_state.key(),
        GameStoreError::ContractAddressMismatch
    );
    require_keys_eq!(
        ctx.accounts.publisher.key(),
        ctx.accounts.pgc_game_state.publisher,
        GameStoreError::Unauthorized
    );

    let store_state = &mut ctx.accounts.store_state;
    store_state.set_discount(&game_id, discount_bps)?;

    emit!(DiscountSet {
        game_id,
        publisher: ctx.accounts.publisher.key(),
        discount_bps,
    });

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use pgc::states::GameState as PgcGameState;
use registry::states::RegistryState;

use crate::{
    constants::{MAX_GAME_ID_LEN, STORE_STATE_SEED},
    errors::GameStoreError,
    events::PriceSet,
    states::StoreState,
};

#[derive(Accounts)]
#[instruction(game_id: String, _price: u64, currency: Pubkey)]
pub struct SetPrice<'info> {
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

    #[account(address = currency)]
    pub currency_mint: InterfaceAccount<'info, Mint>,
}

pub fn handler(
    ctx: Context<SetPrice>,
    game_id: String,
    price: u64,
    currency: Pubkey,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), GameStoreError::EmptyGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, GameStoreError::GameIdTooLong);
    require!(currency != Pubkey::default(), GameStoreError::InvalidCurrency);

    let registry_game = ctx
        .accounts
        .registry_state
        .get_game(&game_id)
        .ok_or(error!(GameStoreError::GameNotFound))?;

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
    require!(ctx.accounts.pgc_game_state.game_id == game_id, GameStoreError::ContractAddressMismatch);
    require_keys_eq!(
        ctx.accounts.currency_mint.key(),
        currency,
        GameStoreError::InvalidPaymentMint
    );

    let store_state = &mut ctx.accounts.store_state;
    store_state.upsert_price(game_id.clone(), price, currency)?;

    emit!(PriceSet {
        game_id,
        publisher: ctx.accounts.publisher.key(),
        price,
        currency,
    });

    Ok(())
}

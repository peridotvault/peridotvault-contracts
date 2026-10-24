use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameFreeFlagUpdated,
    state::GameStoreConfig,
};

/// H3: authority-signed free-flag update.
/// Only the game publisher may flip `is_free`. Setting `true` while active
/// payment options exist is the caller's responsibility to avoid: remove or
/// deactivate options first (adding an active option to a free game is
/// rejected in `set_game_payment_option`). The flag is authoritative at
/// `buy_game` time.
#[derive(Accounts)]
pub struct SetGameFreeFlag<'info> {
    pub publisher: Signer<'info>,
    pub game: Account<'info, pgl1::state::Game>,
    pub registry_game: Account<'info, registry_program::state::RegistryGame>,
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
}

pub(crate) fn handler(ctx: Context<SetGameFreeFlag>, is_free: bool) -> Result<()> {
    require_keys_eq!(
        ctx.accounts.game.publisher,
        ctx.accounts.publisher.key(),
        StoreError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.registry_game.game,
        ctx.accounts.game.key(),
        StoreError::RegistryGameMismatch
    );
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );

    ctx.accounts.game_store_config.is_free = is_free;

    emit!(GameFreeFlagUpdated {
        game: ctx.accounts.game.key(),
        is_free,
    });
    Ok(())
}

use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameFreeFlagUpdated,
    state::GameStoreConfig,
};

/// H3: authority-signed free-flag update.
/// Only the game publisher may flip `is_free`. Setting `true` while any
/// ACTIVE payment option exists is REJECTED (`FreeGameHasPaymentOptions`):
/// the publisher must deactivate (or remove) every option first, then set
/// the flag — the mirror of the `set_game_payment_option` guard that refuses
/// to add an active option to a free game. Together the two guards keep the
/// flag and the option set consistent in both directions, so a game can never
/// be simultaneously "free" (free path open) and "priced" (paid path
/// advertising prices). The flag remains authoritative at `buy_game` time.
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

    // H3 fail-closed mirror guard: `is_free = true` while any ACTIVE payment
    // option exists would open the free path while prices are still listed
    // (paid path blocked, free path open, buyers mint free). Reject it; the
    // publisher must deactivate/remove every option first. The on-chain
    // `active_payment_options` counter (not caller-supplied accounts) makes
    // this un-bypassable by omission. Setting `false` (back to paid) is always
    // allowed; adding options afterwards is gated by the mirror guard.
    if is_free {
        require!(
            ctx.accounts.game_store_config.active_payment_options == 0,
            StoreError::FreeGameHasPaymentOptions
        );
    }

    ctx.accounts.game_store_config.is_free = is_free;

    emit!(GameFreeFlagUpdated {
        game: ctx.accounts.game.key(),
        is_free,
    });
    Ok(())
}

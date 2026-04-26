use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::DiscountSet,
    state::{
        AuthorizedProgram, GameStoreConfig,
    },
};

#[derive(Accounts)]
pub struct SetDiscount<'info> {
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,
    pub source_program: Program<'info, pgl1::program::Pgl1>,
    pub registry_program: Program<'info, registry_program::program::Registry>,
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

pub(crate) fn handler(
    ctx: Context<SetDiscount>,
    discount_bps: Option<u16>,
    discount_starts_at: Option<i64>,
    discount_expires_at: Option<i64>,
) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    require_keys_eq!(ctx.accounts.registry_game.game, ctx.accounts.game.key(), StoreError::RegistryGameMismatch);
    require!(
        matches!(
            ctx.accounts.registry_game.status,
            registry_program::state::GameStatus::Active
        ),
        StoreError::GameNotActive
    );

    if let Some(bps) = discount_bps {
        require!(bps <= 10_000, StoreError::InvalidDiscountBps);
    }
    if let (Some(start), Some(end)) = (discount_starts_at, discount_expires_at) {
        require!(start < end, StoreError::InvalidDiscountWindow);
    }

    let cfg = &mut ctx.accounts.game_store_config;
    cfg.discount_bps = discount_bps;
    cfg.discount_starts_at = discount_starts_at;
    cfg.discount_expires_at = discount_expires_at;

    emit!(DiscountSet {
        game: cfg.game,
        discount_bps,
        discount_starts_at,
        discount_expires_at,
    });
    Ok(())
}

use anchor_lang::prelude::*;

use crate::{errors::StoreError, events::DiscountCleared, state::{AuthorizedSourceProgram, GameStoreConfig, SourceGameMirror}};

#[derive(Accounts)]
pub struct ClearDiscount<'info> {
    pub publisher: Signer<'info>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_source_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedSourceProgram>,
    /// CHECK: trusted program id only
    pub source_program: UncheckedAccount<'info>,
    #[account(owner = source_program.key() @ StoreError::UnsupportedSourceGameOwner)]
    pub game: Account<'info, SourceGameMirror>,
    #[account(
        mut,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
}

pub(crate) fn handler(ctx: Context<ClearDiscount>) -> Result<()> {
    require_keys_eq!(ctx.accounts.game.publisher, ctx.accounts.publisher.key(), StoreError::Unauthorized);
    let cfg = &mut ctx.accounts.game_store_config;
    cfg.discount_bps = None;
    cfg.discount_starts_at = None;
    cfg.discount_expires_at = None;
    emit!(DiscountCleared { game: cfg.game });
    Ok(())
}

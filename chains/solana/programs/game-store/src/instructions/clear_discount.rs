use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::DiscountCleared,
    external::{Pgl1Program, PglGame},
    state::{AuthorizedProgram, GameStoreConfig},
};

#[derive(Accounts)]
pub struct ClearDiscount<'info> {
    pub publisher: &'info Signer,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,
    pub game: &'info Account<PglGame>,
    #[account(
        mut,
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info mut Account<GameStoreConfig>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, ClearDiscount<'info>>,
) -> Result<(), ProgramError> {
    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        StoreError::Unauthorized
    );
    let cfg = &mut ctx.accounts.game_store_config;
    cfg.discount_bps.set(None);
    cfg.discount_starts_at.set(None);
    cfg.discount_expires_at.set(None);
    emit!(DiscountCleared { game: cfg.game })?;
    Ok(())
}

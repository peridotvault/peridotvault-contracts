use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::DiscountSet,
    external::{assert_active_registry_game, Pgl1Program, PglGame, RegistryGame, RegistryProgram},
    instructions::{read_option_i64, read_option_u16},
    state::{AuthorizedProgram, GameStoreConfig},
};

#[derive(Accounts)]
pub struct SetDiscount<'info> {
    pub publisher: &'info Signer,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,
    pub source_program: &'info Program<Pgl1Program>,
    pub registry_program: &'info Program<RegistryProgram>,
    pub game: &'info Account<PglGame>,
    pub registry_game: &'info Account<RegistryGame>,
    #[account(
        mut,
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info mut Account<GameStoreConfig>,
}

pub(crate) fn handler<'info>(ctx: &mut Ctx<'info, SetDiscount<'info>>) -> Result<(), ProgramError> {
    let mut offset = 0usize;
    let discount_bps = read_option_u16(ctx.data, &mut offset)?;
    let discount_starts_at = read_option_i64(ctx.data, &mut offset)?;
    let discount_expires_at = read_option_i64(ctx.data, &mut offset)?;

    require_keys_eq!(
        ctx.accounts.game.publisher()?,
        *ctx.accounts.publisher.address(),
        StoreError::Unauthorized
    );
    require_keys_eq!(
        ctx.accounts.registry_game.game()?,
        *ctx.accounts.game.address(),
        StoreError::RegistryGameMismatch
    );
    assert_active_registry_game(ctx.accounts.registry_game)?;

    if let Some(bps) = discount_bps {
        require!(bps <= 10_000, StoreError::InvalidDiscountBps);
    }
    if let (Some(start), Some(end)) = (discount_starts_at, discount_expires_at) {
        require!(start < end, StoreError::InvalidDiscountWindow);
    }

    let cfg = &mut ctx.accounts.game_store_config;
    cfg.discount_bps.set(discount_bps);
    cfg.discount_starts_at.set(discount_starts_at);
    cfg.discount_expires_at.set(discount_expires_at);

    emit!(DiscountSet {
        game: cfg.game,
        discount_bps_present: discount_bps.is_some(),
        discount_bps: discount_bps.unwrap_or(0),
        discount_starts_at_present: discount_starts_at.is_some(),
        discount_starts_at: discount_starts_at.unwrap_or(0),
        discount_expires_at_present: discount_expires_at.is_some(),
        discount_expires_at: discount_expires_at.unwrap_or(0),
    })?;
    Ok(())
}

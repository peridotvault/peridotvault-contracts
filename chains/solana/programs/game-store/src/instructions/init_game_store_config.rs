use anchor_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GameStoreConfigInitialized,
    state::{AuthorizedProgram, GameStoreConfig, ROLE_REGISTRY},
};

#[derive(Accounts)]
pub struct InitGameStoreConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub publisher: Option<Signer<'info>>,

    pub source_program: Program<'info, pgl1::program::Pgl1>,
    #[account(
        constraint = authorized_source_program.active @ StoreError::SourceProgramNotAuthorized,
        constraint = authorized_source_program.role == 0 @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", source_program.key().as_ref()],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: Account<'info, AuthorizedProgram>,

    pub registry_program: Program<'info, registry_program::program::Registry>,
    #[account(
        constraint = authorized_registry_program.active @ StoreError::RegistryProgramNotAuthorized,
        constraint = authorized_registry_program.role >= ROLE_REGISTRY @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", registry_program.key().as_ref()],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: Account<'info, AuthorizedProgram>,

    pub game: Account<'info, pgl1::state::Game>,
    /// CHECK: registry-owned account; game & status validated manually in handler
    #[account(
        owner = registry_program.key() @ StoreError::RegistryProgramNotAuthorized,
    )]
    pub registry_game: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = GameStoreConfig::SPACE,
        seeds = [b"game_store_config", game.key().as_ref()],
        bump
    )]
    pub game_store_config: Account<'info, GameStoreConfig>,
    pub system_program: Program<'info, System>,
}

pub(crate) fn handler(ctx: Context<InitGameStoreConfig>, active: bool) -> Result<()> {
    let publisher_key = ctx.accounts.game.publisher;

    if let Some(ref publisher) = ctx.accounts.publisher {
        require_keys_eq!(publisher.key(), publisher_key, StoreError::Unauthorized);
    } else {
        require!(
            ctx.accounts.authorized_registry_program.role >= ROLE_REGISTRY,
            StoreError::InsufficientRole
        );
    }

    // Validate: game key matches (registry_game.game written in memory but not yet
    // serialized to account data at CPI time, so skip reading registry_game data)
    require_keys_eq!(ctx.accounts.registry_game.key(), {
        let (pda, _) = Pubkey::find_program_address(
            &[b"registry_game", ctx.accounts.game.key().as_ref()],
            &ctx.accounts.registry_program.key(),
        );
        pda
    }, StoreError::RegistryGameMismatch);

    let cfg = &mut ctx.accounts.game_store_config;
    cfg.game = ctx.accounts.game.key();
    cfg.active = active;
    cfg.referral_bps = None;
    cfg.discount_bps = None;
    cfg.discount_starts_at = None;
    cfg.discount_expires_at = None;
    cfg.bump = ctx.bumps.game_store_config;

    emit!(GameStoreConfigInitialized {
        game: cfg.game,
        active,
    });
    Ok(())
}

use quasar_lang::prelude::*;

use crate::{
    errors::StoreError,
    events::GamePaymentOptionSet,
    external::{assert_active_registry_game, Pgl1Program, PglGame, RegistryGame, RegistryProgram},
    state::{
        AcceptedPaymentToken, AuthorizedProgram, GamePaymentOption, GameStoreConfig, ROLE_REGISTRY,
    },
};

#[derive(Accounts)]
pub struct SetGamePaymentOption<'info> {
    pub payer: &'info mut Signer,
    pub publisher: Option<&'info Signer>,

    pub source_program: &'info Program<Pgl1Program>,
    #[account(
        constraint = authorized_source_program.active.get() @ StoreError::SourceProgramNotAuthorized,
        constraint = authorized_source_program.role == 0 @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", source_program],
        bump = authorized_source_program.bump,
    )]
    pub authorized_source_program: &'info Account<AuthorizedProgram>,

    pub registry_program: &'info Program<RegistryProgram>,
    #[account(
        constraint = authorized_registry_program.active.get() @ StoreError::RegistryProgramNotAuthorized,
        constraint = authorized_registry_program.role >= ROLE_REGISTRY @ StoreError::InsufficientRole,
        seeds = [b"authorized_program", registry_program],
        bump = authorized_registry_program.bump,
    )]
    pub authorized_registry_program: &'info Account<AuthorizedProgram>,

    pub game: &'info Account<PglGame>,
    pub registry_game: &'info Account<RegistryGame>,
    #[account(
        seeds = [b"game_store_config", game],
        bump = game_store_config.bump,
        has_one = game
    )]
    pub game_store_config: &'info Account<GameStoreConfig>,
    pub mint: &'info UncheckedAccount,
    #[account(
        constraint = accepted_payment_token.active.get() @ StoreError::PaymentTokenDisabled,
        seeds = [b"accepted_payment_token", mint],
        bump = accepted_payment_token.bump
    )]
    pub accepted_payment_token: &'info Account<AcceptedPaymentToken>,
    #[account(
        init_if_needed,
        payer = payer,
        space = <GamePaymentOption as Space>::SPACE,
        seeds = [b"game_payment_option", game, mint],
        bump
    )]
    pub game_payment_option: &'info mut Account<GamePaymentOption>,
    pub system_program: &'info Program<System>,
}

pub(crate) fn handler<'info>(
    ctx: &mut Ctx<'info, SetGamePaymentOption<'info>>,
    base_price: u64,
    active: bool,
) -> Result<(), ProgramError> {
    require!(base_price > 0, StoreError::InvalidPrice);

    let publisher_key = ctx.accounts.game.publisher()?;

    if let Some(publisher) = ctx.accounts.publisher {
        require_keys_eq!(
            *publisher.address(),
            publisher_key,
            StoreError::Unauthorized
        );
    } else {
        require!(
            ctx.accounts.authorized_registry_program.role >= ROLE_REGISTRY,
            StoreError::InsufficientRole
        );
    }

    require_keys_eq!(
        ctx.accounts.registry_game.game()?,
        *ctx.accounts.game.address(),
        StoreError::RegistryGameMismatch
    );
    assert_active_registry_game(ctx.accounts.registry_game)?;
    require!(
        ctx.accounts.game_store_config.active.get(),
        StoreError::StoreGameInactive
    );
    require_keys_eq!(
        ctx.accounts.accepted_payment_token.mint,
        *ctx.accounts.mint.address(),
        StoreError::PaymentTokenNotAllowed
    );

    let option = &mut ctx.accounts.game_payment_option;
    if option.game != Address::default() {
        require_keys_eq!(
            option.game,
            *ctx.accounts.game.address(),
            StoreError::GamePaymentOptionMismatch
        );
    }

    option.set_inner(
        *ctx.accounts.game.address(),
        *ctx.accounts.mint.address(),
        base_price,
        active,
        ctx.bumps.game_payment_option,
    );

    emit!(GamePaymentOptionSet {
        game: *ctx.accounts.game.address(),
        mint: *ctx.accounts.mint.address(),
        base_price,
        active,
    })?;
    Ok(())
}

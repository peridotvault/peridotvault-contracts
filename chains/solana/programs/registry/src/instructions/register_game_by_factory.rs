use anchor_lang::prelude::*;
use pgc::states::GameState as PgcGameState;

use crate::{
    constants::{GAME_REGISTRATION_SEED, MAX_GAME_ID_LEN, REGISTRY_STATE_SEED, STATUS_APPROVED, STATUS_PENDING},
    errors::RegistryError,
    events::GameRegistered,
    instructions::collect_registration_fee,
    states::{GameRegistration, RegistryState},
};

#[derive(Accounts)]
#[instruction(game_id: String, contract_address: Pubkey, publisher: Pubkey, _payment_method: Pubkey)]
pub struct RegisterGameByFactory<'info> {
    pub factory: Signer<'info>,

    #[account(mut)]
    pub fee_payer: Signer<'info>,

    #[account(
        mut,
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,

    #[account(address = contract_address)]
    pub pgc_game_state: Account<'info, PgcGameState>,

    /// CHECK: validated against registry_state.treasury
    #[account(mut, address = registry_state.treasury)]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        init,
        payer = fee_payer,
        space = GameRegistration::SPACE,
        seeds = [GAME_REGISTRATION_SEED, game_id.as_bytes()],
        bump
    )]
    pub game_registration: Account<'info, GameRegistration>,

    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub fee_payer_token_account: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub treasury_fee_token_account: Option<UncheckedAccount<'info>>,
    pub fee_payment_mint: Option<UncheckedAccount<'info>>,
    pub token_program: Option<UncheckedAccount<'info>>,
}

pub fn handler(
    ctx: Context<RegisterGameByFactory>,
    game_id: String,
    contract_address: Pubkey,
    publisher: Pubkey,
    payment_method: Pubkey,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::EmptyGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::GameIdTooLong);
    require!(
        contract_address != Pubkey::default(),
        RegistryError::InvalidContractAddress
    );
    require!(publisher != Pubkey::default(), RegistryError::InvalidPublisher);

    let registry_state = &mut ctx.accounts.registry_state;
    require_keys_eq!(
        ctx.accounts.factory.key(),
        registry_state.factory,
        RegistryError::Unauthorized
    );

    let canonical_publisher = ctx.accounts.pgc_game_state.publisher;
    let canonical_game_id = ctx.accounts.pgc_game_state.game_id.clone();

    require_keys_eq!(publisher, canonical_publisher, RegistryError::PublisherMismatch);
    require!(game_id == canonical_game_id, RegistryError::GameIdMismatch);
    let registry_state = &mut ctx.accounts.registry_state;

    let is_fee_exempt = registry_state.is_fee_exempt(&canonical_publisher);

    collect_registration_fee(
        registry_state,
        canonical_publisher,
        payment_method,
        ctx.accounts.fee_payer.to_account_info(),
        ctx.accounts.treasury.to_account_info(),
        ctx.accounts.fee_payer_token_account.as_ref(),
        ctx.accounts.treasury_fee_token_account.as_ref(),
        ctx.accounts.fee_payment_mint.as_ref(),
        ctx.accounts.token_program.as_ref(),
        ctx.accounts.system_program.to_account_info(),
    )?;

    let initial_status = if is_fee_exempt {
        STATUS_APPROVED
    } else {
        STATUS_PENDING
    };

    let game_registration = &mut ctx.accounts.game_registration;
    game_registration.bump = ctx.bumps.game_registration;
    game_registration.game_id = game_id.clone();
    game_registration.contract_address = contract_address;
    game_registration.status = initial_status;

    emit!(GameRegistered {
        game_id,
        contract_address,
        publisher: canonical_publisher,
        status: initial_status,
        registered_by_factory: true,
    });

    Ok(())
}

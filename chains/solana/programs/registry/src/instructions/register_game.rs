use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use pgc::states::GameState as PgcGameState;

use crate::{
    constants::{MAX_GAME_ID_LEN, REGISTRY_STATE_SEED, STATUS_APPROVED, STATUS_PENDING},
    errors::RegistryError,
    events::GameRegistered,
    instructions::collect_registration_fee,
    states::RegistryState,
};

#[derive(Accounts)]
#[instruction(game_id: String, contract_address: Pubkey, _payment_method: Pubkey)]
pub struct RegisterGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

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

    pub publisher_fee_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    pub treasury_fee_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    pub fee_payment_mint: Option<InterfaceAccount<'info, Mint>>,
    pub token_program: Option<Interface<'info, TokenInterface>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterGame>,
    game_id: String,
    contract_address: Pubkey,
    payment_method: Pubkey,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::EmptyGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::GameIdTooLong);
    require!(
        contract_address != Pubkey::default(),
        RegistryError::InvalidContractAddress
    );

    let canonical_publisher = ctx.accounts.pgc_game_state.publisher;
    let canonical_game_id = ctx.accounts.pgc_game_state.game_id.clone();

    require_keys_eq!(
        ctx.accounts.publisher.key(),
        canonical_publisher,
        RegistryError::Unauthorized
    );
    require!(game_id == canonical_game_id, RegistryError::GameIdMismatch);

    let registry_state = &mut ctx.accounts.registry_state;
    require!(
        registry_state.get_game(&game_id).is_none(),
        RegistryError::GameAlreadyRegistered
    );

    let is_fee_exempt = registry_state.is_fee_exempt(&canonical_publisher);

    collect_registration_fee(
        registry_state,
        canonical_publisher,
        payment_method,
        ctx.accounts.publisher.to_account_info(),
        ctx.accounts.treasury.to_account_info(),
        ctx.accounts.publisher_fee_token_account.as_ref(),
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

    registry_state.add_game(game_id.clone(), contract_address, initial_status)?;

    emit!(GameRegistered {
        game_id,
        contract_address,
        publisher: canonical_publisher,
        status: initial_status,
        registered_by_factory: false,
    });

    Ok(())
}

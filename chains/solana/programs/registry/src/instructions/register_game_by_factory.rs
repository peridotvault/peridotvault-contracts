use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use pgc::states::GameState as PgcGameState;

use crate::{
    constants::{MAX_GAME_ID_LEN, REGISTRY_STATE_SEED, STATUS_PENDING},
    errors::RegistryError,
    events::GameRegistered,
    instructions::collect_registration_fee,
    states::RegistryState,
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

    #[account(mut)]
    pub fee_payer_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_fee_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    pub fee_payment_mint: Option<InterfaceAccount<'info, Mint>>,
    pub token_program: Option<Interface<'info, TokenInterface>>,
    pub system_program: Program<'info, System>,
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
    require!(
        registry_state.get_game(&game_id).is_none(),
        RegistryError::GameAlreadyRegistered
    );

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

    registry_state.add_game(game_id.clone(), contract_address, STATUS_PENDING)?;

    emit!(GameRegistered {
        game_id,
        contract_address,
        publisher: canonical_publisher,
        status: STATUS_PENDING,
        registered_by_factory: true,
    });

    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use game_store::{
    cpi as game_store_cpi,
    cpi::accounts::SetPrice as GameStoreSetPriceAccounts,
    program::GameStore,
    states::StoreState,
};
use pgc::{
    cpi as pgc_cpi,
    cpi::accounts::{Initialize as PgcInitializeAccounts, SetMinter as PgcSetMinterAccounts},
    program::Pgc1,
};
use registry::{
    cpi as registry_cpi,
    cpi::accounts::RegisterGameByFactory as RegisterGameByFactoryAccounts,
    program::Registry,
    states::RegistryState,
};
use sha2::{Digest, Sha256};

use crate::{
    constants::{FACTORY_MINT_SEED, FACTORY_STATE_SEED, MAX_GAME_ID_LEN, MAX_METADATA_URI_LEN},
    errors::FactoryError,
    events::GameCreated,
    states::FactoryState,
};

#[derive(Accounts)]
#[instruction(
    game_id: String,
    _metadata_uri: String,
    _initial_price: u64,
    _initial_price_currency: Pubkey,
    _registration_payment_method: Pubkey
)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        seeds = [FACTORY_STATE_SEED],
        bump = factory_state.bump
    )]
    pub factory_state: Account<'info, FactoryState>,

    /// CHECK: Factory derives and signs for this mint PDA, and the PGC CPI initializes and validates it.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,

    pub pgc_program: Program<'info, Pgc1>,

    /// CHECK: validated by the PGC program during CPI
    #[account(mut)]
    pub pgc_game_state: UncheckedAccount<'info>,

    /// CHECK: validated by the PGC program during CPI
    pub pgc_game_authority: UncheckedAccount<'info>,

    /// CHECK: validated by the PGC program during CPI
    #[account(mut)]
    pub publisher_minter_auth: UncheckedAccount<'info>,

    /// CHECK: validated by the PGC program during CPI
    #[account(mut)]
    pub game_store_minter_auth: UncheckedAccount<'info>,

    pub registry_program: Program<'info, Registry>,

    #[account(mut, address = factory_state.registry)]
    pub registry_state: Account<'info, RegistryState>,

    /// CHECK: validated against registry_state.treasury by registry CPI
    #[account(mut, address = registry_state.treasury)]
    pub treasury: UncheckedAccount<'info>,

    #[account(mut, address = factory_state.game_store)]
    pub game_store: Account<'info, StoreState>,

    pub game_store_program: Program<'info, GameStore>,

    #[account(mut)]
    pub publisher_fee_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub treasury_fee_token_account: Option<InterfaceAccount<'info, TokenAccount>>,
    pub fee_payment_mint: Option<InterfaceAccount<'info, Mint>>,
    pub payment_token_program: Option<Interface<'info, TokenInterface>>,
    pub price_currency_mint: Option<InterfaceAccount<'info, Mint>>,

    pub license_token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateGame>,
    game_id: String,
    metadata_uri: String,
    initial_price: u64,
    initial_price_currency: Pubkey,
    registration_payment_method: Pubkey,
) -> Result<Pubkey> {
    require!(!game_id.trim().is_empty(), FactoryError::EmptyGameId);
    require!(!metadata_uri.trim().is_empty(), FactoryError::EmptyMetadataUri);
    require!(game_id.len() <= MAX_GAME_ID_LEN, FactoryError::GameIdTooLong);
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        FactoryError::MetadataUriTooLong
    );

    let mint_hash = Sha256::digest(game_id.as_bytes());
    let (expected_mint, mint_bump) =
        Pubkey::find_program_address(&[FACTORY_MINT_SEED, mint_hash.as_ref()], ctx.program_id);
    require_keys_eq!(ctx.accounts.mint.key(), expected_mint, FactoryError::InvalidMint);
    let mint_signer_seeds: &[&[u8]] = &[FACTORY_MINT_SEED, mint_hash.as_ref(), &[mint_bump]];

    pgc_cpi::initialize(
        CpiContext::new_with_signer(
            ctx.accounts.pgc_program.to_account_info(),
            PgcInitializeAccounts {
                payer: ctx.accounts.publisher.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                game_state: ctx.accounts.pgc_game_state.to_account_info(),
                game_authority: ctx.accounts.pgc_game_authority.to_account_info(),
                publisher_account: ctx.accounts.publisher.to_account_info(),
                publisher_minter_auth: ctx.accounts.publisher_minter_auth.to_account_info(),
                token_program: ctx.accounts.license_token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[mint_signer_seeds],
        ),
        game_id.clone(),
        ctx.accounts.publisher.key(),
        metadata_uri.clone(),
    )?;

    pgc_cpi::set_minter(
        CpiContext::new(
            ctx.accounts.pgc_program.to_account_info(),
            PgcSetMinterAccounts {
                publisher: ctx.accounts.publisher.to_account_info(),
                game_state: ctx.accounts.pgc_game_state.to_account_info(),
                account: ctx.accounts.game_store.to_account_info(),
                minter_auth: ctx.accounts.game_store_minter_auth.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
        ),
        true,
    )?;

    let factory_signer_seeds: &[&[u8]] = &[FACTORY_STATE_SEED, &[ctx.accounts.factory_state.bump]];

    registry_cpi::register_game_by_factory(
        CpiContext::new_with_signer(
            ctx.accounts.registry_program.to_account_info(),
            RegisterGameByFactoryAccounts {
                factory: ctx.accounts.factory_state.to_account_info(),
                fee_payer: ctx.accounts.publisher.to_account_info(),
                registry_state: ctx.accounts.registry_state.to_account_info(),
                pgc_game_state: ctx.accounts.pgc_game_state.to_account_info(),
                treasury: ctx.accounts.treasury.to_account_info(),
                fee_payer_token_account: ctx
                    .accounts
                    .publisher_fee_token_account
                    .as_ref()
                    .map(ToAccountInfo::to_account_info),
                treasury_fee_token_account: ctx
                    .accounts
                    .treasury_fee_token_account
                    .as_ref()
                    .map(ToAccountInfo::to_account_info),
                fee_payment_mint: ctx
                    .accounts
                    .fee_payment_mint
                    .as_ref()
                    .map(ToAccountInfo::to_account_info),
                token_program: ctx
                    .accounts
                    .payment_token_program
                    .as_ref()
                    .map(ToAccountInfo::to_account_info),
                system_program: ctx.accounts.system_program.to_account_info(),
            },
            &[factory_signer_seeds],
        ),
        game_id.clone(),
        ctx.accounts.pgc_game_state.key(),
        ctx.accounts.publisher.key(),
        registration_payment_method,
    )?;

    game_store_cpi::set_price(
        CpiContext::new(
            ctx.accounts.game_store_program.to_account_info(),
            GameStoreSetPriceAccounts {
                publisher: ctx.accounts.publisher.to_account_info(),
                store_state: ctx.accounts.game_store.to_account_info(),
                registry_state: ctx.accounts.registry_state.to_account_info(),
                pgc_game_state: ctx.accounts.pgc_game_state.to_account_info(),
                currency_mint: ctx
                    .accounts
                    .price_currency_mint
                    .as_ref()
                    .map(ToAccountInfo::to_account_info),
            },
        ),
        game_id.clone(),
        initial_price,
        initial_price_currency,
    )?;

    emit!(GameCreated {
        game_id,
        metadata_uri,
        publisher: ctx.accounts.publisher.key(),
        game: ctx.accounts.pgc_game_state.key(),
        mint: ctx.accounts.mint.key(),
    });

    Ok(ctx.accounts.pgc_game_state.key())
}

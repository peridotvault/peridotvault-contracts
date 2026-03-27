use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use pgc::{
    cpi as pgc_cpi,
    cpi::accounts::{Initialize as PgcInitializeAccounts, SetMinter as PgcSetMinterAccounts},
    program::Pgc,
};

use crate::{
    constants::{GAME_SEED, MAX_GAME_ID_LEN, REGISTRY_STATE_SEED, STATUS_APPROVED, STATUS_PENDING},
    errors::RegistryError,
    events::GameRegistered,
    instructions::collect_registration_fee,
    states::{FeeExemptionAccount, GameRegistration, RegistrationFeeOptionAccount, RegistryState},
};

#[derive(Accounts)]
#[instruction(
    game_id: String,
    metadata_uri: String,
    initial_price: u64,
    initial_price_currency: Pubkey,
    registration_payment_method: Pubkey
)]
pub struct PublishGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        seeds = [REGISTRY_STATE_SEED],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,

    /// CHECK: PDA for game mint
    #[account(mut)]
    pub mint: Program<'info, System>,

    pub pgc_program: Program<'info, Pgc>,

    /// CHECK: validated by PGC
    #[account(mut)]
    pub pgc_game_state: Program<'info, System>,

    /// CHECK: validated by PGC
    pub pgc_game_authority: Program<'info, System>,

    /// CHECK: validated by PGC
    #[account(mut)]
    pub publisher_minter_auth: Program<'info, System>,

    /// CHECK: validated by PGC
    #[account(mut)]
    pub game_store_minter_auth: Program<'info, System>,

    #[account(
        init,
        payer = publisher,
        space = GameRegistration::SPACE,
        seeds = [GAME_SEED, game_id.as_bytes()],
        bump
    )]
    pub game_registration: Account<'info, GameRegistration>,

    #[account(
        seeds = [b"fee_option", registration_payment_method.as_ref()],
        bump = fee_option.bump,
    )]
    pub fee_option: Option<Account<'info, RegistrationFeeOptionAccount>>,

    #[account(
        seeds = [b"fee_exemption", publisher.key().as_ref()],
        bump = fee_exemption.bump,
    )]
    pub fee_exemption: Option<Account<'info, FeeExemptionAccount>>,

    /// CHECK: validated against registry_state.treasury
    #[account(mut, address = registry_state.treasury)]
    pub treasury: Program<'info, System>,

    /// CHECK: validated by game_store program address
    pub game_store_program: Program<'info, System>,
    
    /// CHECK: validated by game_store CPI
    #[account(mut)]
    pub store_state: Program<'info, System>,

    /// CHECK: validated by game_store CPI
    #[account(mut)]
    pub price_account: Program<'info, System>,

    /// CHECK: validated by game_store CPI
    pub price_currency_mint: Option<Program<'info, System>>,

    pub license_token_program: Program<'info, Token2022>,
    /// CHECK: system program bypass
    pub sys_prog: Program<'info, System>,

    #[account(mut)]
    pub publisher_fee_token_account: Option<Program<'info, System>>,
    #[account(mut)]
    pub treasury_fee_token_account: Option<Program<'info, System>>,
    pub fee_payment_mint: Option<Program<'info, System>>,
    pub payment_token_program: Option<Program<'info, System>>,
}

pub fn handler(
    ctx: Context<PublishGame>,
    game_id: String,
    metadata_uri: String,
    initial_price: u64,
    initial_price_currency: Pubkey,
    registration_payment_method: Pubkey,
) -> Result<()> {
    require!(!game_id.trim().is_empty(), RegistryError::EmptyGameId);
    require!(game_id.len() <= MAX_GAME_ID_LEN, RegistryError::GameIdTooLong);

    // 1. Initialize PGC
    pgc_cpi::initialize(
        CpiContext::new(
            ctx.accounts.pgc_program.to_account_info(),
            PgcInitializeAccounts {
                payer: ctx.accounts.publisher.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                game_state: ctx.accounts.pgc_game_state.to_account_info(),
                game_authority: ctx.accounts.pgc_game_authority.to_account_info(),
                publisher_account: ctx.accounts.publisher.to_account_info(),
                publisher_minter_auth: ctx.accounts.publisher_minter_auth.to_account_info(),
                token_program: ctx.accounts.license_token_program.to_account_info(),
                system_program: ctx.accounts.sys_prog.to_account_info(),
            },
        ),
        game_id.clone(),
        ctx.accounts.publisher.key(),
        metadata_uri.clone(),
    )?;

    // 2. Auth GameStore to mint licenses
    pgc_cpi::set_minter(
        CpiContext::new(
            ctx.accounts.pgc_program.to_account_info(),
            PgcSetMinterAccounts {
                publisher: ctx.accounts.publisher.to_account_info(),
                game_state: ctx.accounts.pgc_game_state.to_account_info(),
                account: ctx.accounts.store_state.to_account_info(),
                minter_auth: ctx.accounts.game_store_minter_auth.to_account_info(),
                system_program: ctx.accounts.sys_prog.to_account_info(),
            },
        ),
        true,
    )?;

    // 3. Set Price in GameStore (Account-based)
    // We use raw CPI to break cyclic dependency
    let mut data = Vec::with_capacity(8 + 4 + game_id.len() + 8 + 32);
    // sighash for "set_price"
    data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:set_price").to_bytes()[..8]);
    // args: game_id: String, price: u64, currency: Pubkey
    data.extend_from_slice(&(game_id.len() as u32).to_le_bytes());
    data.extend_from_slice(game_id.as_bytes());
    data.extend_from_slice(&initial_price.to_le_bytes());
    data.extend_from_slice(&initial_price_currency.to_bytes());

    let mut accounts = vec![
        AccountMeta::new(ctx.accounts.publisher.key(), true),
        AccountMeta::new_readonly(ctx.accounts.store_state.key(), false),
        AccountMeta::new_readonly(ctx.accounts.registry_state.key(), false),
        AccountMeta::new_readonly(ctx.accounts.pgc_game_state.key(), false),
        AccountMeta::new(ctx.accounts.price_account.key(), false),
        AccountMeta::new_readonly(ctx.accounts.game_registration.key(), false),
        AccountMeta::new_readonly(ctx.accounts.sys_prog.key(), false),
    ];

    if let Some(mint) = &ctx.accounts.price_currency_mint {
        accounts.push(AccountMeta::new_readonly(mint.key(), false));
    }

    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.accounts.game_store_program.key(),
        accounts,
        data,
    };

    let mut account_infos = vec![
        ctx.accounts.publisher.to_account_info(),
        ctx.accounts.store_state.to_account_info(),
        ctx.accounts.registry_state.to_account_info(),
        ctx.accounts.pgc_game_state.to_account_info(),
        ctx.accounts.price_account.to_account_info(),
        ctx.accounts.game_registration.to_account_info(),
        ctx.accounts.sys_prog.to_account_info(),
    ];
    if let Some(mint) = &ctx.accounts.price_currency_mint {
        account_infos.push(mint.to_account_info());
    }

    anchor_lang::solana_program::program::invoke(
        &ix,
        &account_infos,
    )?;

    // 4. Registry internal registration (collect fee)
    let is_fee_exempt = ctx.accounts.fee_exemption.is_some();

    collect_registration_fee(
        &ctx.accounts.registry_state,
        ctx.accounts.fee_option.clone(),
        is_fee_exempt,
        ctx.accounts.publisher.to_account_info(),
        ctx.accounts.treasury.to_account_info(),
        ctx.accounts.publisher_fee_token_account.as_ref(),
        ctx.accounts.treasury_fee_token_account.as_ref(),
        ctx.accounts.fee_payment_mint.as_ref(),
        ctx.accounts.payment_token_program.as_ref(),
        ctx.accounts.sys_prog.to_account_info(),
    )?;

    let initial_status = if is_fee_exempt {
        STATUS_APPROVED
    } else {
        STATUS_PENDING
    };

    let game_registration = &mut ctx.accounts.game_registration;
    game_registration.bump = ctx.bumps.game_registration;
    game_registration.game_id = game_id.clone();
    game_registration.contract_address = ctx.accounts.pgc_game_state.key();
    game_registration.status = initial_status;

    emit!(GameRegistered {
        game_id,
        contract_address: ctx.accounts.pgc_game_state.key(),
        publisher: ctx.accounts.publisher.key(),
        status: initial_status,
        registered_by_factory: true,
    });

    Ok(())
}

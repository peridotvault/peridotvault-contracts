use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RegistryError;

// --- Initialize ---

pub fn initialize_handler(ctx: Context<Initialize>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.authority.key();
    config.registration_fee = 10_000_000; // 0.01 SOL default
    config.bump = ctx.bumps.config;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = RegistryConfig::SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, RegistryConfig>,
    pub system_program: Program<'info, System>,
}

// --- Register Game ---

pub fn register_game_handler(
    ctx: Context<RegisterGame>,
    game_id: String,
    pgc_program: Pubkey,
    pgc_game: Pubkey,
) -> Result<()> {
    // 1. Pay Fee
    let fee = ctx.accounts.config.registration_fee;
    if fee > 0 {
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.publisher.key(),
                &ctx.accounts.treasury.key(),
                fee,
            ),
            &[
                ctx.accounts.publisher.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    // 2. State
    let game = &mut ctx.accounts.game;
    game.game_id = game_id;
    game.publisher = ctx.accounts.publisher.key();
    game.pgc_pid = pgc_program;
    game.pgc_pda = pgc_game;
    game.active = true;
    game.created_at = Clock::get()?.unix_timestamp;
    game.bump = ctx.bumps.game;

    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct RegisterGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, RegistryConfig>,
    /// CHECK: Treasury from config
    #[account(
        mut,
        constraint = treasury.key() == config.treasury @ RegistryError::InvalidAuthority
    )]
    pub treasury: UncheckedAccount<'info>,
    #[account(
        init,
        payer = publisher,
        space = RegistryGameAccount::SPACE,
        seeds = [b"game", game_id.as_bytes()],
        bump
    )]
    pub game: Account<'info, RegistryGameAccount>,
    pub system_program: Program<'info, System>,
}

// --- Update Game ---

pub fn update_game_handler(
    ctx: Context<UpdateGame>,
    _game_id: String,
    pgc_program: Pubkey,
    pgc_game: Pubkey,
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.pgc_pid = pgc_program;
    game.pgc_pda = pgc_game;
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct UpdateGame<'info> {
    pub publisher: Signer<'info>,
    #[account(
        mut,
        seeds = [b"game", game_id.as_bytes()],
        bump = game.bump,
        has_one = publisher @ RegistryError::Unauthorized
    )]
    pub game: Account<'info, RegistryGameAccount>,
}

// --- Set Status ---

pub fn set_status_handler(ctx: Context<SetStatus>, _game_id: String, active: bool) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.active = active;
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct SetStatus<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,
    #[account(
        mut,
        seeds = [b"game", game_id.as_bytes()],
        bump = game.bump
    )]
    pub game: Account<'info, RegistryGameAccount>,
}

// --- Set Registration Fee ---

pub fn set_registration_fee_handler(ctx: Context<SetRegistrationFee>, fee: u64) -> Result<()> {
    ctx.accounts.config.registration_fee = fee;
    Ok(())
}

#[derive(Accounts)]
pub struct SetRegistrationFee<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,
}

// --- Set Treasury ---

pub fn set_treasury_handler(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
    ctx.accounts.config.treasury = treasury;
    Ok(())
}

#[derive(Accounts)]
pub struct SetTreasury<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority @ RegistryError::Unauthorized
    )]
    pub config: Account<'info, RegistryConfig>,
}

// --- Transfer Publisher ---

pub fn transfer_publisher_handler(
    ctx: Context<TransferPublisher>,
    _game_id: String,
    new_publisher: Pubkey,
) -> Result<()> {
    ctx.accounts.game.publisher = new_publisher;
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct TransferPublisher<'info> {
    pub publisher: Signer<'info>,
    #[account(
        mut,
        seeds = [b"game", game_id.as_bytes()],
        bump = game.bump,
        has_one = publisher @ RegistryError::Unauthorized
    )]
    pub game: Account<'info, RegistryGameAccount>,
}

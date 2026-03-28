use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(game_id: String, metadata_uri: String, initial_minter: Pubkey)]
pub struct CreateGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        init,
        payer = publisher,
        space = PgcGameAccount::SPACE,
        seeds = [SEED_GAME, game_id.as_bytes()],
        bump
    )]
    pub game_account: Account<'info, PgcGameAccount>,

    #[account(
        init,
        payer = publisher,
        space = MinterAccount::SPACE,
        seeds = [SEED_MINTER, game_account.key().as_ref(), initial_minter.as_ref()],
        bump
    )]
    pub initial_minter_account: Account<'info, MinterAccount>,

    /// CHECK: Registry Program
    pub registry_program: UncheckedAccount<'info>,
    /// CHECK: Store Program
    pub store_program: UncheckedAccount<'info>,

    /// CHECK: Registry Game Account
    #[account(mut)]
    pub registry_game: UncheckedAccount<'info>,

    /// CHECK: Price Account
    #[account(mut)]
    pub price_account: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintLicense<'info> {
    #[account(mut)]
    pub minter: Signer<'info>,

    #[account(
        seeds = [SEED_MINTER, game.key().as_ref(), minter.key().as_ref()],
        bump,
        constraint = minter_account.is_authorized @ crate::errors::PgcError::Unauthorized
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub game: Account<'info, PgcGameAccount>,

    /// CHECK: The user receiving the license
    pub user: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = minter,
        space = LicenseAccount::SPACE,
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(account: Pubkey)]
pub struct SetMinter<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        has_one = publisher @ crate::errors::PgcError::Unauthorized
    )]
    pub game_account: Account<'info, PgcGameAccount>,

    #[account(
        init_if_needed,
        payer = publisher,
        space = MinterAccount::SPACE,
        seeds = [SEED_MINTER, game_account.key().as_ref(), account.as_ref()],
        bump
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    pub minter: Signer<'info>,

    #[account(
        seeds = [SEED_MINTER, game.key().as_ref(), minter.key().as_ref()],
        bump,
        constraint = minter_account.is_authorized @ crate::errors::PgcError::Unauthorized
    )]
    pub minter_account: Account<'info, MinterAccount>,

    pub game: Account<'info, PgcGameAccount>,

    /// CHECK: The user to revoke
    pub user: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}

#[derive(Accounts)]
pub struct UpdateMetadataUri<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,

    #[account(
        mut,
        has_one = publisher @ crate::errors::PgcError::Unauthorized
    )]
    pub game_account: Account<'info, PgcGameAccount>,
}

#[derive(Accounts)]
pub struct HasLicense<'info> {
    pub game: Account<'info, PgcGameAccount>,
    /// CHECK: The user to check
    pub user: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}

#[derive(Accounts)]
pub struct CanAccessGame<'info> {
    pub game: Account<'info, PgcGameAccount>,
    /// CHECK: The user to check
    pub user: UncheckedAccount<'info>,
    #[account(
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}


use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;
pub mod utils;

pub use constants::*;
pub use errors::*;
pub use events::*;
pub use state::*;
pub use instructions::*;

declare_id!("3ZbX4ehgZYZ6TXARcF8tVsJmjxNoB5D67PkXiXqjk1JA");

#[program]
pub mod pgc1 {
    use super::*;

    pub fn create_game(
        ctx: Context<CreateGame>, 
        game_id: String, 
        metadata_uri: String, 
        mint: Option<Pubkey>
    ) -> Result<()> {
        instructions::create_game_handler(ctx, game_id, metadata_uri, mint)
    }

    pub fn issue_license(
        ctx: Context<IssueLicense>, 
        expires_at: i64
    ) -> Result<()> {
        instructions::issue_license_handler(ctx, expires_at)
    }

    pub fn revoke_license(ctx: Context<RevokeLicense>) -> Result<()> {
        instructions::revoke_license_handler(ctx)
    }

    pub fn assert_license(ctx: Context<AssertLicense>) -> Result<()> {
        instructions::assert_license_handler(ctx)
    }
}

#[derive(Accounts)]
#[instruction(game_id: String)]
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

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IssueLicense<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub game: Account<'info, PgcGameAccount>,
    
    /// CHECK: User to receive license
    pub user: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = LicenseAccount::SPACE,
        seeds = [SEED_LICENSE, user.key().as_ref(), game.key().as_ref()],
        bump
    )]
    pub license_account: Account<'info, LicenseAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    
    #[account(has_one = publisher @ PgcError::Unauthorized)]
    pub game: Account<'info, PgcGameAccount>,
    
    #[account(
        mut,
        close = publisher,
        seeds = [SEED_LICENSE, license_account.owner.as_ref(), game.key().as_ref()],
        bump = license_account.bump
    )]
    pub license_account: Account<'info, LicenseAccount>,
}

#[derive(Accounts)]
pub struct AssertLicense<'info> {
    pub license_account: Account<'info, LicenseAccount>,
}

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;

pub use constants::*;
pub use errors::*;
pub use events::*;
pub use state::*;
pub use instructions::*;

declare_id!("CrHcKzUfp9ykApDFt5tBzs1MK41QAjbrPVxdoYifzE2r");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        authority: Pubkey,
    ) -> Result<()> {
        crate::instructions::initialize_handler(ctx, authority)
    }

    pub fn register_game(
        ctx: Context<RegisterGame>,
        game_id: String,
        pgc_program: Pubkey,
        pgc_game: Pubkey,
    ) -> Result<()> {
        crate::instructions::register_game_handler(ctx, game_id, pgc_program, pgc_game)
    }

    pub fn update_game(
        ctx: Context<UpdateGame>,
        pgc_program: Pubkey,
        pgc_game: Pubkey,
    ) -> Result<()> {
        crate::instructions::update_game_handler(ctx, pgc_program, pgc_game)
    }

    pub fn set_status(
        ctx: Context<SetStatus>,
        active: bool,
    ) -> Result<()> {
        crate::instructions::set_status_handler(ctx, active)
    }

    pub fn transfer_publisher(
        ctx: Context<TransferPublisher>,
        new_publisher: Pubkey,
    ) -> Result<()> {
        crate::instructions::transfer_publisher_handler(ctx, new_publisher)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init, 
        payer = payer, 
        space = state::RegistryConfig::SPACE, 
        seeds = [REGISTRY_CONFIG_SEED], 
        bump
    )]
    pub registry_config: Account<'info, state::RegistryConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(game_id: String)]
pub struct RegisterGame<'info> {
    #[account(mut)]
    pub publisher: Signer<'info>,
    #[account(
        init, 
        payer = publisher, 
        space = state::RegistryGameAccount::SPACE, 
        seeds = [GAME_SEED, game_id.as_bytes()], 
        bump
    )]
    pub game_account: Account<'info, state::RegistryGameAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGame<'info> {
    pub publisher: Signer<'info>,
    #[account(
        mut, 
        seeds = [GAME_SEED, game_account.game_id.as_bytes()], 
        bump = game_account.bump,
        has_one = publisher @ RegistryError::Unauthorized
    )]
    pub game_account: Account<'info, state::RegistryGameAccount>,
}

#[derive(Accounts)]
pub struct SetStatus<'info> {
    pub authority: Signer<'info>,
    #[account(seeds = [REGISTRY_CONFIG_SEED], bump = registry_config.bump, has_one = authority @ RegistryError::Unauthorized)]
    pub registry_config: Account<'info, state::RegistryConfig>,
    #[account(
        mut, 
        seeds = [GAME_SEED, game_account.game_id.as_bytes()], 
        bump = game_account.bump
    )]
    pub game_account: Account<'info, state::RegistryGameAccount>,
}

#[derive(Accounts)]
pub struct TransferPublisher<'info> {
    pub publisher: Signer<'info>,
    #[account(
        mut, 
        seeds = [GAME_SEED, game_account.game_id.as_bytes()], 
        bump = game_account.bump,
        has_one = publisher @ RegistryError::Unauthorized
    )]
    pub game_account: Account<'info, state::RegistryGameAccount>,
}

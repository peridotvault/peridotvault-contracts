use anchor_lang::prelude::*;

pub mod state;
pub mod errors;
pub mod instructions;

pub use state::*;
pub use errors::*;
pub use instructions::*;

declare_id!("DCYPxPtnVeBgy56SYMT6GPBMJp8NJNLmE46QfHYqCgGL");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx)
    }

    pub fn register_game(ctx: Context<RegisterGame>, game_id: String, pgc_program: Pubkey, pgc_game: Pubkey) -> Result<()> {
        register_game_handler(ctx, game_id, pgc_program, pgc_game)
    }

    pub fn update_game(ctx: Context<UpdateGame>, game_id: String, pgc_program: Pubkey, pgc_game: Pubkey) -> Result<()> {
        update_game_handler(ctx, game_id, pgc_program, pgc_game)
    }

    pub fn set_status(ctx: Context<SetStatus>, game_id: String, active: bool) -> Result<()> {
        set_status_handler(ctx, game_id, active)
    }

    pub fn set_registration_fee(ctx: Context<SetRegistrationFee>, fee: u64) -> Result<()> {
        set_registration_fee_handler(ctx, fee)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        set_treasury_handler(ctx, treasury)
    }

    pub fn transfer_publisher(ctx: Context<TransferPublisher>, game_id: String, new_publisher: Pubkey) -> Result<()> {
        transfer_publisher_handler(ctx, game_id, new_publisher)
    }
}

use anchor_lang::prelude::*;

pub mod state;
pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod utils;

pub use state::*;
pub use errors::*;
pub use events::*;

declare_id!("3ZbX4ehgZYZ6TXARcF8tVsJmjxNoB5D67PkXiXqjk1JA");

#[program]
pub mod pgc1 {
    use super::*;

    /// Initializes a new PgcGameAccount and bootstraps Registry/Store via CPI.
    /// Maps to original PGC-1 standard `initialize`.
    pub fn create_game(
        ctx: Context<CreateGame>,
        game_id: String,
        metadata_uri: String,
        initial_minter: Pubkey,
        price: u64,
        currency: Pubkey,
    ) -> Result<()> {
        instructions::create_game::create_game_handler(ctx, game_id, metadata_uri, initial_minter, price, currency)
    }

    /// Grants or upgrades a user license. Callable only by authorized minters.
    /// Maps to original PGC-1 standard `mintLicense`.
    pub fn mint_license(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
        instructions::mint_license::handler(ctx, expires_at)
    }

    /// Authorizes or deauthorizes a minter for a specific game. Only publisher.
    /// Maps to original PGC-1 standard `setMinter`.
    pub fn set_minter(ctx: Context<SetMinter>, account: Pubkey, is_authorized: bool) -> Result<()> {
        instructions::set_minter::handler(ctx, account, is_authorized)
    }

    /// Revokes a user license by setting expiry to now. Only authorized minters.
    pub fn revoke_license(ctx: Context<RevokeLicense>) -> Result<()> {
        instructions::revoke_license::handler(ctx)
    }

    /// Updates game metadata URI. Only publisher.
    pub fn update_metadata_uri(ctx: Context<UpdateMetadataUri>, new_uri: String) -> Result<()> {
        instructions::update_metadata_uri::handler(ctx, new_uri)
    }

    // --- View Semantics ---

    /// Returns whether user has a valid license.
    /// Maps to original PGC-1 standard `hasLicense`.
    pub fn has_license(ctx: Context<HasLicense>) -> Result<bool> {
        instructions::has_license::handler(ctx)
    }

    /// Returns whether user can access the game (currrently mirrors has_license).
    /// Maps to original PGC-1 standard `canAccessGame`.
    pub fn can_access_game(ctx: Context<CanAccessGame>) -> Result<bool> {
        instructions::can_access_game::handler(ctx)
    }
}

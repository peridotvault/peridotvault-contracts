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
pub use instructions::*;

declare_id!("DzDbFZXZsmFFv1mMFimLaBjAQi7Z5gUaQ61qcDuR6Kor");

#[program]
pub mod pgc1 {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, authority: Pubkey, authorized_store: Pubkey) -> Result<()> {
        initialize_handler(ctx, authority, authorized_store)
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        game_id: String,
        metadata_uri: String,
        initial_minter: Pubkey,
        price: u64,
        currency: Pubkey,
    ) -> Result<()> {
        create_game_handler(ctx, game_id, metadata_uri, initial_minter, price, currency)
    }

    pub fn mint_license(ctx: Context<MintLicense>, expires_at: i64) -> Result<()> {
        mint_license_handler(ctx, expires_at)
    }

    pub fn set_minter(ctx: Context<SetMinter>, minter: Pubkey, enabled: bool) -> Result<()> {
        set_minter_handler(ctx, minter, enabled)
    }

    pub fn revoke_license(ctx: Context<RevokeLicense>) -> Result<()> {
        revoke_license_handler(ctx)
    }

    pub fn update_metadata_uri(ctx: Context<UpdateMetadataUri>, new_uri: String) -> Result<()> {
        update_metadata_uri_handler(ctx, new_uri)
    }

    pub fn set_publisher(ctx: Context<SetPublisher>, new_publisher: Pubkey) -> Result<()> {
        set_publisher_handler(ctx, new_publisher)
    }

    pub fn has_license(ctx: Context<HasLicense>) -> Result<bool> {
        has_license_handler(ctx)
    }

    pub fn can_access_game(ctx: Context<CanAccessGame>) -> Result<bool> {
        can_access_game_handler(ctx)
    }
}

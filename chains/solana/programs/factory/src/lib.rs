use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::{
    create_game::CreateGame,
    initialize::Initialize,
    set_game_store::SetGameStore,
    set_governance::SetGovernance,
    set_registry::SetRegistry,
    views::GetFactoryView,
};
use instructions::{
    create_game::__client_accounts_create_game,
    initialize::__client_accounts_initialize,
    set_game_store::__client_accounts_set_game_store,
    set_governance::__client_accounts_set_governance,
    set_registry::__client_accounts_set_registry,
    views::__client_accounts_get_factory_view,
};

declare_id!("3EaXmAr9wAvYgXhz1BH4Kpa5DDCc5oTykeeGtBHeqYXA");

#[program]
pub mod factory {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        governance: Pubkey,
        registry: Pubkey,
        game_store: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, governance, registry, game_store)
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        game_id: String,
        metadata_uri: String,
        registration_payment_method: Pubkey,
    ) -> Result<Pubkey> {
        instructions::create_game::handler(ctx, game_id, metadata_uri, registration_payment_method)
    }

    pub fn set_registry(ctx: Context<SetRegistry>, registry: Pubkey) -> Result<()> {
        instructions::set_registry::handler(ctx, registry)
    }

    pub fn set_game_store(ctx: Context<SetGameStore>, game_store: Pubkey) -> Result<()> {
        instructions::set_game_store::handler(ctx, game_store)
    }

    pub fn set_governance(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
        instructions::set_governance::handler(ctx, governance)
    }

    pub fn get_registry(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
        instructions::views::get_registry(ctx)
    }

    pub fn get_game_store(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
        instructions::views::get_game_store(ctx)
    }

    pub fn get_governance(ctx: Context<GetFactoryView>) -> Result<Pubkey> {
        instructions::views::get_governance(ctx)
    }
}

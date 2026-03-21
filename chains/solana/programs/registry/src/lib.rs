use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod states;

pub use instructions::{
    initialize::Initialize,
    register_game::RegisterGame,
    register_game_by_factory::RegisterGameByFactory,
    set_admin::SetAdmin,
    set_factory::SetFactory,
    set_fee_exemption::SetFeeExemption,
    set_governance::SetGovernance,
    set_registration_fee::SetRegistrationFee,
    set_status::SetStatus,
    set_treasury::SetTreasury,
    views::{GetRegistryView, RegistrationFeeView},
};
pub use states::{RegistrationFeeOption, RegistryGame};
#[allow(unused_imports)]
use instructions::{
    initialize::__cpi_client_accounts_initialize,
    initialize::__client_accounts_initialize,
    register_game::__cpi_client_accounts_register_game,
    register_game::__client_accounts_register_game,
    register_game_by_factory::__cpi_client_accounts_register_game_by_factory,
    register_game_by_factory::__client_accounts_register_game_by_factory,
    set_admin::__cpi_client_accounts_set_admin,
    set_admin::__client_accounts_set_admin,
    set_factory::__cpi_client_accounts_set_factory,
    set_factory::__client_accounts_set_factory,
    set_fee_exemption::__cpi_client_accounts_set_fee_exemption,
    set_fee_exemption::__client_accounts_set_fee_exemption,
    set_governance::__cpi_client_accounts_set_governance,
    set_governance::__client_accounts_set_governance,
    set_registration_fee::__cpi_client_accounts_set_registration_fee,
    set_registration_fee::__client_accounts_set_registration_fee,
    set_status::__cpi_client_accounts_set_status,
    set_status::__client_accounts_set_status,
    set_treasury::__cpi_client_accounts_set_treasury,
    set_treasury::__client_accounts_set_treasury,
    views::__cpi_client_accounts_get_registry_view,
    views::__client_accounts_get_registry_view,
};

declare_id!("3bUSqLjWxUgmruzuRwhtWwhV93b4RXVN7bE5qHxHHxLj");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        governance: Pubkey,
        treasury: Pubkey,
        factory: Pubkey,
        registration_fee: u64,
        registration_fee_token: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(
            ctx,
            governance,
            treasury,
            factory,
            registration_fee,
            registration_fee_token,
        )
    }

    pub fn register_game(
        ctx: Context<RegisterGame>,
        game_id: String,
        contract_address: Pubkey,
        payment_method: Pubkey,
    ) -> Result<()> {
        instructions::register_game::handler(ctx, game_id, contract_address, payment_method)
    }

    pub fn register_game_by_factory(
        ctx: Context<RegisterGameByFactory>,
        game_id: String,
        contract_address: Pubkey,
        publisher: Pubkey,
        payment_method: Pubkey,
    ) -> Result<()> {
        instructions::register_game_by_factory::handler(
            ctx,
            game_id,
            contract_address,
            publisher,
            payment_method,
        )
    }

    pub fn set_status(ctx: Context<SetStatus>, game_id: String, status: u8) -> Result<()> {
        instructions::set_status::handler(ctx, game_id, status)
    }

    pub fn set_admin(ctx: Context<SetAdmin>, account: Pubkey, is_admin: bool) -> Result<()> {
        instructions::set_admin::handler(ctx, account, is_admin)
    }

    pub fn set_governance(ctx: Context<SetGovernance>, governance: Pubkey) -> Result<()> {
        instructions::set_governance::handler(ctx, governance)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        instructions::set_treasury::handler(ctx, treasury)
    }

    pub fn set_factory(ctx: Context<SetFactory>, factory: Pubkey) -> Result<()> {
        instructions::set_factory::handler(ctx, factory)
    }

    pub fn set_registration_fee(
        ctx: Context<SetRegistrationFee>,
        amount: u64,
        token: Pubkey,
    ) -> Result<()> {
        instructions::set_registration_fee::handler(ctx, amount, token)
    }

    pub fn set_fee_exemption(
        ctx: Context<SetFeeExemption>,
        account: Pubkey,
        is_exempt: bool,
    ) -> Result<()> {
        instructions::set_fee_exemption::handler(ctx, account, is_exempt)
    }

    pub fn get_game(ctx: Context<GetRegistryView>, game_id: String) -> Result<RegistryGame> {
        instructions::views::get_game(ctx, game_id)
    }

    pub fn get_all_games(ctx: Context<GetRegistryView>) -> Result<Vec<RegistryGame>> {
        instructions::views::get_all_games(ctx)
    }

    pub fn get_contract_address(
        ctx: Context<GetRegistryView>,
        game_id: String,
    ) -> Result<Pubkey> {
        instructions::views::get_contract_address(ctx, game_id)
    }

    pub fn get_status(ctx: Context<GetRegistryView>, game_id: String) -> Result<u8> {
        instructions::views::get_status(ctx, game_id)
    }

    pub fn get_governance(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
        instructions::views::get_governance(ctx)
    }

    pub fn get_treasury(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
        instructions::views::get_treasury(ctx)
    }

    pub fn get_factory(ctx: Context<GetRegistryView>) -> Result<Pubkey> {
        instructions::views::get_factory(ctx)
    }

    pub fn get_registration_fee(
        ctx: Context<GetRegistryView>,
        payment_method: Pubkey,
    ) -> Result<RegistrationFeeView> {
        instructions::views::get_registration_fee(ctx, payment_method)
    }

    pub fn get_registration_fees(
        ctx: Context<GetRegistryView>,
    ) -> Result<Vec<RegistrationFeeView>> {
        instructions::views::get_registration_fees(ctx)
    }

    pub fn is_fee_exempt(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
        instructions::views::is_fee_exempt(ctx, account)
    }

    pub fn is_admin(ctx: Context<GetRegistryView>, account: Pubkey) -> Result<bool> {
        instructions::views::is_admin(ctx, account)
    }
}

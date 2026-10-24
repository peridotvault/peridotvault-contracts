use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod xchain;
pub mod xchain_attest;

pub use instructions::*;

declare_id!("G2XvhJoEkjiu3rCysaAjTuDj1dT5NAS8RNUTVi9H7ggE");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize_registry(ctx: Context<InitializeRegistry>, treasury: Pubkey) -> Result<()> {
        instructions::initialize_registry::handler(ctx, treasury)
    }

    pub fn set_treasury(ctx: Context<SetTreasury>, treasury: Pubkey) -> Result<()> {
        instructions::set_treasury::handler(ctx, treasury)
    }

    pub fn add_payment_token(ctx: Context<AddPaymentToken>, fee_amount: u64) -> Result<()> {
        instructions::add_payment_token::handler(ctx, fee_amount)
    }

    pub fn update_payment_token(
        ctx: Context<UpdatePaymentToken>,
        active: bool,
        fee_amount: u64,
    ) -> Result<()> {
        instructions::update_payment_token::handler(ctx, active, fee_amount)
    }

    pub fn remove_payment_token(ctx: Context<RemovePaymentToken>) -> Result<()> {
        instructions::remove_payment_token::handler(ctx)
    }

    pub fn create_publish_grant(ctx: Context<CreatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
        instructions::set_publish_grant::create_handler(ctx, expired_at)
    }

    pub fn update_publish_grant(ctx: Context<UpdatePublishGrant>, expired_at: Option<i64>) -> Result<()> {
        instructions::set_publish_grant::update_handler(ctx, expired_at)
    }

    pub fn create_game_and_register<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateGameAndRegister<'info>>,
        game_id: String,
        metadata_uri: String,
        base_price: Option<u64>,
        mint_token: Option<Pubkey>,
    ) -> Result<()> {
        instructions::create_game_and_register::handler(ctx, game_id, metadata_uri, base_price, mint_token)
    }

    pub fn update_game_status(ctx: Context<UpdateGameStatus>, status: u8) -> Result<()> {
        instructions::update_game_status::handler(ctx, status)
    }

    pub fn close_registry_game(ctx: Context<CloseRegistryGame>) -> Result<()> {
        instructions::close_registry_game::handler(ctx)
    }

    // M1 cross-chain entries (ANT-490).
    pub fn propose_xchain_game(
        ctx: Context<ProposeXchainGame>,
        chain_id: String,
        publisher: Vec<u8>,
        local_id: String,
        program_id: Vec<u8>,
        format: u8,
        game_id: [u8; 32],
    ) -> Result<()> {
        instructions::xchain_entry::propose_handler(ctx, chain_id, publisher, local_id, program_id, format, game_id)
    }

    pub fn activate_xchain_game(ctx: Context<XchainEntryAdmin>) -> Result<()> {
        instructions::xchain_entry::activate_handler(ctx)
    }

    pub fn deprecate_xchain_game(ctx: Context<XchainEntryAdmin>) -> Result<()> {
        instructions::xchain_entry::deprecate_handler(ctx)
    }

    pub fn freeze_xchain_game(ctx: Context<XchainEntryAdmin>) -> Result<()> {
        instructions::xchain_entry::freeze_handler(ctx)
    }

    pub fn propose_xchain_update(
        ctx: Context<XchainEntryAdmin>,
        new_program_id: Vec<u8>,
        timelock_delay: i64,
    ) -> Result<()> {
        instructions::xchain_entry::propose_update_handler(ctx, new_program_id, timelock_delay)
    }

    pub fn cancel_xchain_update(ctx: Context<XchainEntryAdmin>) -> Result<()> {
        instructions::xchain_entry::cancel_update_handler(ctx)
    }

    pub fn execute_xchain_update(ctx: Context<XchainEntryAdmin>) -> Result<()> {
        instructions::xchain_entry::execute_update_handler(ctx)
    }

    pub fn cancel_xchain_proposal(ctx: Context<CloseXchainProposal>) -> Result<()> {
        instructions::xchain_entry::cancel_proposal_handler(ctx)
    }

    // M2 attestation consume path (ANT-487, recognition-only).
    pub fn init_xchain_attester(
        ctx: Context<InitXchainAttester>,
        signers: [[u8; 20]; 5],
        reorg_delay: i64,
    ) -> Result<()> {
        instructions::xchain_consume::init_handler(ctx, signers, reorg_delay)
    }

    pub fn update_xchain_attester(
        ctx: Context<UpdateXchainAttester>,
        signers: [[u8; 20]; 5],
        reorg_delay: i64,
    ) -> Result<()> {
        instructions::xchain_consume::update_handler(ctx, signers, reorg_delay)
    }

    pub fn consume_xchain_attestation(
        ctx: Context<ConsumeXchainAttestation>,
        args: ConsumeArgs,
    ) -> Result<()> {
        instructions::xchain_consume::consume_handler(ctx, args)
    }
}

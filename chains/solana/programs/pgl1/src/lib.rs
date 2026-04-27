#![no_std]
#![allow(unexpected_cfgs)]
#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
extern crate std;

use quasar_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::{
    AddAuthorizedActor, CloseAuthorizedActor, CloseCreatorState, CreateGame,
    DeactivateAuthorizedActor, InitializePgl, MintLicense, RenewLicense, SetAuthority,
    SetCreateGameFee, SetMetadataUri, SetPublisher, SetTreasury,
};

pub use state::*;

declare_id!("5YctJfQJ6qfYDchYKyHFyjeKa3dx8Z6kg5pt68yaZ6c3");

#[program]
pub mod pgl1 {
    use super::*;

    #[instruction(discriminator = [27, 26, 103, 210, 20, 202, 72, 9])]
    pub fn initialize_pgl(
        ctx: Ctx<InitializePgl>,
        treasury: Address,
        create_game_fee_lamports: u64,
    ) -> Result<(), ProgramError> {
        instructions::initialize_pgl::handler(&mut ctx, treasury, create_game_fee_lamports)
    }

    #[instruction(discriminator = [180, 158, 35, 163, 88, 50, 43, 157])]
    pub fn set_create_game_fee(
        ctx: Ctx<SetCreateGameFee>,
        create_game_fee_lamports: u64,
    ) -> Result<(), ProgramError> {
        instructions::set_create_game_fee::handler(&mut ctx, create_game_fee_lamports)
    }

    #[instruction(discriminator = [57, 97, 196, 95, 195, 206, 106, 136])]
    pub fn set_treasury(ctx: Ctx<SetTreasury>, treasury: Address) -> Result<(), ProgramError> {
        instructions::set_treasury::handler(&mut ctx, treasury)
    }

    #[instruction(discriminator = [133, 250, 37, 21, 110, 163, 26, 121])]
    pub fn set_authority(
        ctx: Ctx<SetAuthority>,
        new_authority: Address,
    ) -> Result<(), ProgramError> {
        instructions::set_authority::handler(&mut ctx, new_authority)
    }

    #[instruction(discriminator = [36, 250, 169, 0, 167, 155, 131, 155])]
    pub fn add_authorized_actor(ctx: Ctx<AddAuthorizedActor>) -> Result<(), ProgramError> {
        instructions::add_authorized_actor::handler(&mut ctx)
    }

    #[instruction(discriminator = [180, 68, 94, 97, 242, 126, 165, 142])]
    pub fn deactivate_authorized_actor(
        ctx: Ctx<DeactivateAuthorizedActor>,
    ) -> Result<(), ProgramError> {
        instructions::deactivate_authorized_actor::handler(&mut ctx)
    }

    #[instruction(discriminator = [127, 140, 77, 23, 48, 163, 227, 117])]
    pub fn close_authorized_actor(ctx: Ctx<CloseAuthorizedActor>) -> Result<(), ProgramError> {
        instructions::close_authorized_actor::handler(&mut ctx)
    }

    #[instruction(discriminator = [130, 195, 143, 56, 69, 235, 205, 164])]
    pub fn close_creator_state(ctx: Ctx<CloseCreatorState>) -> Result<(), ProgramError> {
        instructions::close_creator_state::handler(&mut ctx)
    }

    #[instruction(discriminator = [124, 69, 75, 66, 184, 220, 72, 206])]
    pub fn create_game(ctx: Ctx<CreateGame>) -> Result<(), ProgramError> {
        instructions::create_game::handler(&mut ctx)
    }

    #[instruction(discriminator = [110, 54, 4, 216, 151, 85, 46, 91])]
    pub fn set_publisher(
        ctx: Ctx<SetPublisher>,
        new_publisher: Address,
    ) -> Result<(), ProgramError> {
        instructions::set_publisher::handler(&mut ctx, new_publisher)
    }

    #[instruction(discriminator = [30, 134, 3, 67, 40, 90, 245, 34])]
    pub fn set_metadata_uri(ctx: Ctx<SetMetadataUri>) -> Result<(), ProgramError> {
        instructions::set_metadata_uri::handler(&mut ctx)
    }

    #[instruction(discriminator = [57, 204, 93, 84, 160, 241, 254, 52])]
    pub fn mint_license(ctx: Ctx<MintLicense>) -> Result<(), ProgramError> {
        instructions::mint_license::handler(&mut ctx)
    }

    #[instruction(discriminator = [104, 243, 122, 253, 203, 203, 199, 64])]
    pub fn renew_license(ctx: Ctx<RenewLicense>, expires_at: i64) -> Result<(), ProgramError> {
        instructions::renew_license::handler(&mut ctx, expires_at)
    }
}

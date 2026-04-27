#![no_std]
#![allow(unexpected_cfgs)]
#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
extern crate std;

use quasar_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod external;
pub mod instructions;
pub mod state;

use instructions::{
    AddPaymentToken, CloseRegistryGame, CreateGameAndRegister, CreatePublishGrant,
    InitializeRegistry, RemovePaymentToken, SetTreasury, UpdateGameStatus, UpdatePaymentToken,
    UpdatePublishGrant,
};

declare_id!("8pgmtQDVpMX4FHmoCmWJCoB94RY56GKWUzo8f8e1Xfpo");

#[program]
pub mod registry {
    use super::*;
    #[instruction(discriminator = [189, 181, 20, 17, 174, 57, 249, 59])]
    pub fn initialize_registry(
        ctx: Ctx<InitializeRegistry>,
        treasury: Address,
    ) -> Result<(), ProgramError> {
        instructions::initialize_registry::handler(&mut ctx, treasury)
    }
    #[instruction(discriminator = [57, 97, 196, 95, 195, 206, 106, 136])]
    pub fn set_treasury(ctx: Ctx<SetTreasury>, treasury: Address) -> Result<(), ProgramError> {
        instructions::set_treasury::handler(&mut ctx, treasury)
    }
    #[instruction(discriminator = [19, 203, 48, 148, 80, 1, 179, 140])]
    pub fn add_payment_token(
        ctx: Ctx<AddPaymentToken>,
        fee_amount: u64,
    ) -> Result<(), ProgramError> {
        instructions::add_payment_token::handler(&mut ctx, fee_amount)
    }
    #[instruction(discriminator = [240, 107, 161, 243, 84, 148, 183, 126])]
    pub fn update_payment_token(
        ctx: Ctx<UpdatePaymentToken>,
        active: bool,
        fee_amount: u64,
    ) -> Result<(), ProgramError> {
        instructions::update_payment_token::handler(&mut ctx, active, fee_amount)
    }
    #[instruction(discriminator = [119, 18, 240, 223, 126, 168, 165, 117])]
    pub fn remove_payment_token(ctx: Ctx<RemovePaymentToken>) -> Result<(), ProgramError> {
        instructions::remove_payment_token::handler(&mut ctx)
    }
    #[instruction(discriminator = [83, 236, 41, 49, 105, 161, 61, 173])]
    pub fn create_publish_grant(ctx: Ctx<CreatePublishGrant>) -> Result<(), ProgramError> {
        instructions::set_publish_grant::create_handler(&mut ctx)
    }
    #[instruction(discriminator = [185, 215, 123, 61, 37, 3, 134, 206])]
    pub fn update_publish_grant(ctx: Ctx<UpdatePublishGrant>) -> Result<(), ProgramError> {
        instructions::set_publish_grant::update_handler(&mut ctx)
    }
    #[instruction(discriminator = [78, 43, 148, 255, 70, 148, 207, 218])]
    pub fn create_game_and_register(
        ctx: CtxWithRemaining<CreateGameAndRegister>,
    ) -> Result<(), ProgramError> {
        instructions::create_game_and_register::handler(&mut ctx)
    }
    #[instruction(discriminator = [31, 175, 127, 242, 51, 244, 172, 185])]
    pub fn update_game_status(ctx: Ctx<UpdateGameStatus>, status: u8) -> Result<(), ProgramError> {
        instructions::update_game_status::handler(&mut ctx, status)
    }
    #[instruction(discriminator = [137, 24, 5, 199, 115, 15, 111, 244])]
    pub fn close_registry_game(ctx: Ctx<CloseRegistryGame>) -> Result<(), ProgramError> {
        instructions::close_registry_game::handler(&mut ctx)
    }
}

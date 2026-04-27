#![no_std]
#[cfg(not(any(target_os = "solana", target_arch = "bpf")))]
extern crate std;

use quasar_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod external;
pub mod instructions;
pub mod state;

use instructions::{
    AddAuthorizedProgram, AddPaymentToken, BuyGame, ClearDiscount, InitGameStoreConfig,
    InitializeStore, RemoveGamePaymentOption, SetDefaultReferral, SetDiscount,
    SetGamePaymentOption, SetGameStoreActive, SetMaxReferral, SetPlatformFee, SetReferralBps,
    SetStoreActor, SetTreasury, UpdateAuthorizedProgram, UpdatePaymentToken,
};

declare_id!("8xi62uARkmBcKKwG3M8uvFnaayZL4MFvkQ91WG16eBCj");

#[program]
pub mod game_store {
    use super::*;

    #[instruction(discriminator = [109, 149, 210, 214, 188, 126, 220, 140])]
    pub fn initialize_store(
        ctx: Ctx<InitializeStore>,
        treasury: Address,
        platform_fee_bps: u16,
        default_referral_bps: u16,
        max_referral_bps: u16,
        store_actor: Address,
    ) -> Result<(), ProgramError> {
        instructions::initialize_store::handler(
            &mut ctx,
            treasury,
            platform_fee_bps,
            default_referral_bps,
            max_referral_bps,
            store_actor,
        )
    }

    #[instruction(discriminator = [57, 97, 196, 95, 195, 206, 106, 136])]
    pub fn set_treasury(ctx: Ctx<SetTreasury>, treasury: Address) -> Result<(), ProgramError> {
        instructions::set_treasury::handler(&mut ctx, treasury)
    }

    #[instruction(discriminator = [19, 70, 111, 182, 156, 58, 208, 203])]
    pub fn set_platform_fee(
        ctx: Ctx<SetPlatformFee>,
        platform_fee_bps: u16,
    ) -> Result<(), ProgramError> {
        instructions::set_platform_fee::handler(&mut ctx, platform_fee_bps)
    }

    #[instruction(discriminator = [17, 210, 30, 108, 163, 67, 215, 80])]
    pub fn set_default_referral(
        ctx: Ctx<SetDefaultReferral>,
        default_referral_bps: u16,
    ) -> Result<(), ProgramError> {
        instructions::set_default_referral::handler(&mut ctx, default_referral_bps)
    }

    #[instruction(discriminator = [136, 204, 233, 199, 249, 248, 137, 144])]
    pub fn set_max_referral(
        ctx: Ctx<SetMaxReferral>,
        max_referral_bps: u16,
    ) -> Result<(), ProgramError> {
        instructions::set_max_referral::handler(&mut ctx, max_referral_bps)
    }

    #[instruction(discriminator = [80, 106, 127, 205, 217, 53, 202, 202])]
    pub fn add_authorized_program(
        ctx: Ctx<AddAuthorizedProgram>,
        role: u8,
    ) -> Result<(), ProgramError> {
        instructions::add_authorized_program::handler(&mut ctx, role)
    }

    #[instruction(discriminator = [70, 84, 196, 221, 239, 138, 173, 238])]
    pub fn update_authorized_program(
        ctx: Ctx<UpdateAuthorizedProgram>,
    ) -> Result<(), ProgramError> {
        instructions::update_authorized_program::handler(&mut ctx)
    }

    #[instruction(discriminator = [19, 203, 48, 148, 80, 1, 179, 140])]
    pub fn add_payment_token(ctx: Ctx<AddPaymentToken>) -> Result<(), ProgramError> {
        instructions::add_payment_token::handler(&mut ctx)
    }

    #[instruction(discriminator = [240, 107, 161, 243, 84, 148, 183, 126])]
    pub fn update_payment_token(
        ctx: Ctx<UpdatePaymentToken>,
        active: bool,
    ) -> Result<(), ProgramError> {
        instructions::update_payment_token::handler(&mut ctx, active)
    }

    #[instruction(discriminator = [85, 106, 133, 8, 211, 20, 78, 108])]
    pub fn init_game_store_config(
        ctx: Ctx<InitGameStoreConfig>,
        active: bool,
    ) -> Result<(), ProgramError> {
        instructions::init_game_store_config::handler(&mut ctx, active)
    }

    #[instruction(discriminator = [89, 147, 76, 11, 9, 108, 85, 219])]
    pub fn set_game_store_active(
        ctx: Ctx<SetGameStoreActive>,
        active: bool,
    ) -> Result<(), ProgramError> {
        instructions::set_game_store_active::handler(&mut ctx, active)
    }

    #[instruction(discriminator = [122, 86, 158, 12, 148, 161, 8, 46])]
    pub fn set_game_payment_option(
        ctx: Ctx<SetGamePaymentOption>,
        base_price: u64,
        active: bool,
    ) -> Result<(), ProgramError> {
        instructions::set_game_payment_option::handler(&mut ctx, base_price, active)
    }

    #[instruction(discriminator = [25, 2, 53, 66, 23, 57, 219, 154])]
    pub fn remove_game_payment_option(
        ctx: Ctx<RemoveGamePaymentOption>,
    ) -> Result<(), ProgramError> {
        instructions::remove_game_payment_option::handler(&mut ctx)
    }

    #[instruction(discriminator = [185, 99, 11, 85, 175, 2, 42, 198])]
    pub fn set_discount(ctx: Ctx<SetDiscount>) -> Result<(), ProgramError> {
        instructions::set_discount::handler(&mut ctx)
    }

    #[instruction(discriminator = [131, 52, 86, 51, 205, 130, 233, 36])]
    pub fn clear_discount(ctx: Ctx<ClearDiscount>) -> Result<(), ProgramError> {
        instructions::clear_discount::handler(&mut ctx)
    }

    #[instruction(discriminator = [28, 213, 164, 214, 151, 184, 143, 136])]
    pub fn set_referral_bps(ctx: Ctx<SetReferralBps>) -> Result<(), ProgramError> {
        instructions::set_referral_bps::handler(&mut ctx)
    }

    #[instruction(discriminator = [52, 118, 95, 161, 244, 179, 250, 38])]
    pub fn set_store_actor(
        ctx: Ctx<SetStoreActor>,
        new_store_actor: Address,
    ) -> Result<(), ProgramError> {
        instructions::set_store_actor::handler(&mut ctx, new_store_actor)
    }

    #[instruction(discriminator = [230, 118, 208, 28, 185, 30, 230, 155])]
    pub fn buy_game(ctx: Ctx<BuyGame>) -> Result<(), ProgramError> {
        instructions::buy_game::handler(&mut ctx)
    }
}

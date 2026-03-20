use anchor_lang::prelude::*;

declare_id!("3EaXmAr9wAvYgXhz1BH4Kpa5DDCc5oTykeeGtBHeqYXA");

#[program]
pub mod factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

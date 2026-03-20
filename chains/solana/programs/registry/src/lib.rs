use anchor_lang::prelude::*;

declare_id!("3bUSqLjWxUgmruzuRwhtWwhV93b4RXVN7bE5qHxHHxLj");

#[program]
pub mod registry {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

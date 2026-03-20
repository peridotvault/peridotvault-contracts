use anchor_lang::prelude::*;

declare_id!("DSiyompbYR2k2GsS69FWkvE9N3vf32Da4JNqZKYvn2Pp");

#[program]
pub mod solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

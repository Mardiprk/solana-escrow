use anchor_lang::prelude::*;

declare_id!("75HqDxF2QFqe7gbSyFP6YKCbVReWxAWtxhD4UgZtqkqh");

#[program]
pub mod escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

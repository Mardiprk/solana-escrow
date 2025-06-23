use anchor_lang::prelude::*;
use anchor_lang::solana_program;

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


#[account]
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub arbiter: Option<Pubkey>,
    pub amount: u64,
    pub expires_at: i64,
    pub state: EscrowState,
    pub escrow_id: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EscrowState {
    Created,
    Active,
    Apprived,
    Completed,
    Cancelled,
    Refunded,
    Cancelled,
}

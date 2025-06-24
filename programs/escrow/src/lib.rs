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
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = buyer,
        space = 8 + EscrowAccount::INIT_SPACE,
        seeds = [ESCROW_SEED, buyer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReleaseFunds<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, buyer.key().as_ref()],
        bump = escrow.bump,
        has_one = buyer @ ErrorCode::Unauthorized,
        has_one = seller @ ErrorCode::InvalidSeller,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub buyer: Signer<'info>,

    #[account(mut)]
    pub seller: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RefundEscrow<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, buyer.key().as_ref()],
        bump = escrow.bump,
        has_one = buyer @ ErrorCode::Unauthorized,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,
}

#[account]
#[derive(InitSpace)] // this will automatically know the size of the account data struct for us
pub struct EscrowAccount {
    pub buyer: Pubkey,      // 32 bytes
    pub seller: Pubkey,     // 32 bytes
    pub amount: u64,        // 8 bytes, because 64/8 = 8
    pub state: EscrowState, // 1 byte
    pub bump: u8,           // 1 byte
}

// the escrow state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum EscrowState {
    Active,
    Completed,
    Refund,
}

// Custom error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount: must be greater than 0")]
    InvalidAmount,
    #[msg("Invalid state: escrow is not active")]
    InvalidState,
    #[msg("Unauthorized: only buyer can perform this action")]
    Unauthorized,
    #[msg("Invalid seller: seller address mismatch")]
    InvalidSeller,
    #[msg("Insufficient funds in escrow")]
    InsufficientFunds,
}

use anchor_lang::prelude::*;
use anchor_lang::solana_program;

declare_id!("75HqDxF2QFqe7gbSyFP6YKCbVReWxAWtxhD4UgZtqkqh");

const ESCROW_SEED: &[u8] = b"escrow";
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

#[program]
pub mod escrow {
    use super::*;

    pub fn create_escrow(ctx: Context<CreateEscrow>, seller: Pubkey, amount: u64) -> Result<()> {
        // Validate that the amount is positive
        require!(amount > 0, ErrorCode::InvalidAmount);

        let escrow = &mut ctx.accounts.escrow;

        // Initialize escrow account data
        escrow.buyer = ctx.accounts.buyer.key();
        escrow.seller = seller;
        escrow.amount = amount;
        escrow.state = EscrowState::Active;
        escrow.bump = ctx.bumps.escrow;

        // Transfer SOL from buyer to escrow PDA
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.escrow.to_account_info(),
                },
            ),
            amount,
        )?;

        msg!(
            "Escrow Created: {} SOL locked for seller {}",
            amount as f64 / LAMPORTS_PER_SOL as f64,
            seller
        );

        Ok(())
    }

    pub fn release_funds(ctx: Context<ReleaseFunds>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.state == EscrowState::Active,
            ErrorCode::InvalidState
        );

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            ErrorCode::Unauthorized
        );

        // Update escrow state to completed
        escrow.state = EscrowState::Completed;

        // Get current escrow balance
        let escrow_balance = ctx.accounts.escrow.to_account_info().lamports();

        // Ensure escrow has sufficient funds
        require!(
            escrow_balance >= escrow.amount,
            ErrorCode::InsufficientFunds
        );

        // Transfer funds from escrow PDA to seller
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += escrow.amount;

        msg!(
            "Funds Released: {} SOL transferred to seller {}",
            escrow.amount as f64 / LAMPORTS_PER_SOL as f64,
            escrow.seller
        );

        Ok(())
    }

    /// Refunds escrowed funds back to the buyer
    pub fn refund_escrow(ctx: Context<RefundEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.state == EscrowState::Active,
            ErrorCode::InvalidState
        );

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            ErrorCode::Unauthorized
        );

        escrow.state = EscrowState::Refunded;

        let escrow_balance = ctx.accounts.escrow.to_account_info().lamports();

        require!(
            escrow_balance >= escrow.amount,
            ErrorCode::InsufficientFunds
        );

        // Transfer funds from escrow PDA back to buyer
        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;
        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += escrow.amount;

        msg!(
            "Escrow Refunded: {} SOL returned to buyer {}",
            escrow.amount as f64 / LAMPORTS_PER_SOL as f64,
            escrow.buyer
        );

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;

        require!(
            escrow.state == EscrowState::Active,
            ErrorCode::InvalidState
        );

        require!(
            ctx.accounts.buyer.key() == escrow.buyer,
            ErrorCode::Unauthorized
        );

        escrow.state = EscrowState::Cancelled;

        let escrow_balance = ctx.accounts.escrow.to_account_info().lamports();
        require!(
            escrow_balance >= escrow.amount,
            ErrorCode::InsufficientFunds
        );

        **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= escrow.amount;
        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += escrow.amount;

        msg!(
            "Escrow Cancelled: {} SOL returned to buyer {}",
            escrow.amount as f64 / LAMPORTS_PER_SOL as f64,
            escrow.buyer
        );

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
        bump,
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

// Account validation for refunding escrow
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

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
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
#[derive(InitSpace)]
pub struct EscrowAccount {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub state: EscrowState,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum EscrowState {
    Active,
    Completed,
    Refunded,
    Cancelled,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid amount: must be greater than 0")]
    InvalidAmount,
    #[msg("Invalid state: operation not allowed in current escrow state")]
    InvalidState,
    #[msg("Unauthorized: only the buyer can perform this action")]
    Unauthorized,
    #[msg("Invalid seller: seller address does not match escrow")]
    InvalidSeller,
    #[msg("Insufficient funds: escrow does not have enough lamports")]
    InsufficientFunds,
}
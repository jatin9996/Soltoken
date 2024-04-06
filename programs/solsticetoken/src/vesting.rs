use anchor_lang::prelude::*;
use crate::state::{UserState, RewardEntitlement};
use anchor_spl::token::{self, Transfer, TokenAccount, Token};
use crate::mlm::MLMStrategy; 

declare_id!("...");

#[program]
pub mod solsticetoken_vesting {
    use super::*;

    pub fn initialize_vesting(ctx: Context<InitializeVesting>, user: Pubkey, amount: u64) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        vesting_account.user = user;
        vesting_account.amount_purchased = amount;
        vesting_account.vesting_start_timestamp = Clock::get()?.unix_timestamp;
        // Set vesting end timestamp to 2 years from the start
        vesting_account.vesting_end_timestamp = vesting_account.vesting_start_timestamp + (2 * 365 * 24 * 60 * 60);
        vesting_account.rewards_claimed = 0;

        // Initialize MLM participation
        MLMStrategy::initialize_participant(ctx.accounts.mlm_participant.to_account_info(), user)?;

        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let now = Clock::get()?.unix_timestamp;

        if now < vesting_account.vesting_end_timestamp {
            return Err(ErrorCode::VestingPeriodNotCompleted.into());
        }

        let total_rewards = vesting_account.amount_purchased * 40; // Example reward calculation
        let rewards_to_claim = total_rewards - vesting_account.rewards_claimed;

        // Ensure rewards have not already been claimed
        if rewards_to_claim <= 0 {
            return Err(ErrorCode::NoRewardsAvailable.into());
        }

        vesting_account.rewards_claimed += rewards_to_claim;

        // Additional logic to calculate MLM rewards
        let mlm_rewards = MLMStrategy::calculate_rewards(ctx.accounts.mlm_participant.to_account_info())?;
        let total_rewards = rewards_to_claim + mlm_rewards;

        // Transfer total rewards (vesting + MLM) to the user
        let cpi_accounts = Transfer {
            from: ctx.accounts.solhit_token_source.to_account_info(),
            to: ctx.accounts.user_solhit_token_account.to_account_info(),
            authority: ctx.accounts.vesting_account_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.solhit_token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, total_rewards)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVesting<'info> {
    #[account(init, payer = user, space = 8 + 8 + 8 + 8 + 8 + 8)]
    pub vesting_account: Account<'info, UserState>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is safe because...
    pub mlm_participant: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub vesting_account: Account<'info, UserState>,
    /// CHECK: This is safe because...
    pub solhit_token_source: AccountInfo<'info>,
    #[account(mut)]
    pub user_solhit_token_account: Account<'info, TokenAccount>,
    pub vesting_account_authority: Signer<'info>,
    pub solhit_token_program: Program<'info, Token>,
    /// CHECK: This is safe because...
    pub mlm_participant: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The vesting period has not been completed.")]
    VestingPeriodNotCompleted,
    #[msg("No rewards available to claim.")]
    NoRewardsAvailable,
}

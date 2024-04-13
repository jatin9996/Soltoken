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

    pub fn update_referral_rewards(ctx: Context<UpdateReferralRewards>, referred_user: Pubkey) -> Result<()> {
        let user_state = &mut ctx.accounts.user_state;
        let referral_reward = 10; 

        
        if user_state.referred_by == Some(referred_user) {
            user_state.rewards_claimed += referral_reward;
        } else {
           
            return Err(ErrorCode::InvalidReferral.into());
        }

        Ok(())
    }

    pub fn update_locked_amount_and_rewards(ctx: Context<UpdateLockedAmountAndRewards>, additional_amount: u64) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let now = Clock::get()?.unix_timestamp;

        // Calculate the time elapsed since the last reward calculation
        let time_elapsed = now - vesting_account.last_reward_calculation_timestamp;

        // Calculate existing rewards (this is a placeholder, implement your own logic)
        let existing_rewards = (vesting_account.amount_purchased * time_elapsed as u64) / (365 * 24 * 60 * 60); // Example calculation

        // Update the locked amount
        vesting_account.amount_purchased += additional_amount;

        // Reset the reward calculation mechanism
        vesting_account.last_reward_calculation_timestamp = now;

        // Optionally, update the rewards_claimed to include the newly calculated rewards
        vesting_account.rewards_claimed += existing_rewards;

        Ok(())
    }

    pub fn update_reward(ctx: Context<UpdateReward>, additional_amount: u64) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let now = Clock::get()?.unix_timestamp;

        let time_elapsed = now - vesting_account.last_reward_calculation_timestamp.max(vesting_account.vesting_start_timestamp);

        let annual_interest_rate = 0.05; 

        let years_elapsed = time_elapsed as f64 / (365.0 * 24.0 * 60.0 * 60.0);

        let existing_rewards = (vesting_account.amount_purchased as f64 * (1.0 + annual_interest_rate).powf(years_elapsed)) - vesting_account.amount_purchased as f64;

        vesting_account.amount_purchased += additional_amount;

        vesting_account.rewards_claimed += existing_rewards as u64;

        // Reset the reward calculation mechanism
        vesting_account.last_reward_calculation_timestamp = now;

        Ok(())
    }

    pub fn view_rewards(ctx: Context<ViewRewards>, user: Pubkey) -> Result<u64> {
        let vesting_account = &ctx.accounts.vesting_account;
        let now = Clock::get()?.unix_timestamp;

        // Calculate the time elapsed since the last reward calculation or since the tokens were locked
        let time_elapsed = now - vesting_account.last_reward_calculation_timestamp.max(vesting_account.vesting_start_timestamp);

        let annual_interest_rate = 0.05; // Clearly defined reward calculation mechanism

        // Calculate the number of years elapsed for the interest calculation
        let years_elapsed = time_elapsed as f64 / (365.0 * 24.0 * 60.0 * 60.0);

        // Calculate existing rewards based on the amount locked and the time elapsed
        let existing_rewards = (vesting_account.amount_purchased as f64 * (1.0 + annual_interest_rate).powf(years_elapsed)) - vesting_account.amount_purchased as f64;

        // Return the calculated rewards, rounded down to the nearest whole number as u64
        Ok(existing_rewards as u64)
    }

    // Adjust the determine_sale_level function to determine the sale level based on the current timestamp
    fn determine_sale_level(now: i64) -> u8 {
        let sale_start_timestamp = /* timestamp for the start of the sale */;
        let level_duration = 15 * 24 * 60 * 60; // 15 days in seconds
        let elapsed_time = now - sale_start_timestamp;
        let current_level = elapsed_time / level_duration;

        match current_level {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            _ => 5, // Default to level 5 if the time exceeds the duration of the first four levels
        }
    }

    // Adjust the calculate_pledge_tokens function to calculate the number of Pledge tokens based on the USD amount and sale level
    fn calculate_pledge_tokens(amount_usd: u64, sale_level: u8) -> u64 {
        match sale_level {
            1 => amount_usd * 2,
            2 => amount_usd * 175 / 100,
            3 => amount_usd * 15 / 10,
            4 => amount_usd * 125 / 100,
            5 => amount_usd, 
            _ => 0, 
        }
    }

    pub fn purchase_and_vest(ctx: Context<PurchaseAndVest>, amount_usd: u64) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let sale_level = determine_sale_level(now);
        let pledge_tokens = calculate_pledge_tokens(amount_usd, sale_level);
    
        mint_pledge_tokens(ctx.accounts.pledge_token_mint.to_account_info(), ctx.accounts.user_pledge_token_account.to_account_info(), pledge_tokens)?;
    
        // Initialize vesting
        let vesting_account = &mut ctx.accounts.vesting_account;
        vesting_account.user = *ctx.accounts.user.key;
        vesting_account.amount_purchased += pledge_tokens;
        vesting_account.vesting_start_timestamp = now;
        // Set vesting end timestamp to 2 years from the start
        vesting_account.vesting_end_timestamp = now + (2 * 365 * 24 * 60 * 60);
        vesting_account.rewards_claimed = 0;
    
        // Initialize MLM participation
        MLMStrategy::initialize_participant(ctx.accounts.mlm_participant.to_account_info(), *ctx.accounts.user.key)?;
    
        add_distribution_event(&mut pledge_distribution, *ctx.accounts.user.key, pledge_tokens);
    
        Ok(())
    }

    fn add_distribution_event(distribution_array: &mut Vec<DistributionEvent>, recipient: Pubkey, amount: u64) {
        let event = DistributionEvent { recipient, amount };
        distribution_array.push(event);
    }

    fn determine_sale_level(now: i64) -> u8 {
        let sale_start_timestamp = /* timestamp for the start of the sale */;
        let level_duration = 15 * 24 * 60 * 60; // 15 days in seconds
        let elapsed_time = now - sale_start_timestamp;
        let current_level = elapsed_time / level_duration;

        match current_level {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            _ => 5, // Default to level 5 if the time exceeds the duration of the first four levels
        }
    }

    fn calculate_pledge_tokens(amount_usd: u64, sale_level: u8) -> u64 {
        match sale_level {
            1 => amount_usd * 2,
            2 => amount_usd * 175 / 100,
            3 => amount_usd * 15 / 10,
            4 => amount_usd * 125 / 100,
            5 => amount_usd, // Level 5 offers 1 Pledge token per $1
            _ => 0, // Default case if the sale level is not recognized
        }
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

#[derive(Accounts)]
pub struct UpdateReferralRewards<'info> {
    #[account(mut)]
    pub user_state: Account<'info, UserState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateLockedAmountAndRewards<'info> {
    #[account(mut)]
    pub vesting_account: Account<'info, UserState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateReward<'info> {
    #[account(mut)]
    pub vesting_account: Account<'info, UserState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ViewRewards<'info> {
    pub vesting_account: Account<'info, UserState>,
}
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct DistributionEvent {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("The vesting period has not been completed.")]
    VestingPeriodNotCompleted,
    #[msg("No rewards available to claim.")]
    NoRewardsAvailable,
    #[msg("Invalid referral.")]
    InvalidReferral,
}

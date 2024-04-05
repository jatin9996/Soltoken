mod error;
mod state;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use error::MyContractError;
use state::{ICOState, PhaseDetail, UserState};

declare_id!("HmKiLcNDNciqozcGuZshomJ72HmkuGmDoVGoLChPJFag");

#[program]
pub mod solsticetoken {

    use super::*;

    pub fn start_ico(
        ctx: Context<StartIco>,
        start_timestamp: i64,
        total_tokens_allocated: u64,
        phase_details: Vec<PhaseDetail>,
    ) -> Result<()> {
        let ico_state = &mut ctx.accounts.ico_state;
        ico_state.start_timestamp = start_timestamp;
        ico_state.total_tokens_allocated = total_tokens_allocated;
        ico_state.phase_details = phase_details;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct StartIco<'info> {
    #[account(init, payer = admin, space = 10240)]
    pub ico_state: Account<'info, ICOState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeSolhitToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, mint::decimals = 9, mint::authority = admin)]
    pub solhit_token_mint: Account<'info, Mint>, // SolhitToken mint account
    #[account(init, payer = admin, token::mint = solhit_token_mint, token::authority = vesting_contract)]
    pub vesting_contract_account: Account<'info, TokenAccount>, // Account to hold 4 million tokens for rewards
    #[account(init, payer = admin, token::mint = solhit_token_mint, token::authority = admin)]
    pub distribution_account: Account<'info, TokenAccount>, // For the 10 million tokens distribution
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeSolhitToken<'info> {
    pub fn execute(
        ctx: Context<InitializeSolhitToken>,
        exchange_or_other_destination: Pubkey,
    ) -> Result<()> {
        // Define the amounts for minting
        let rewards_amount: u64 =
            4_000_000 * 10u64.pow(ctx.accounts.solhit_token_mint.decimals as u32); // 4 million tokens
        let distribution_amount: u64 =
            10_000_000 * 10u64.pow(ctx.accounts.solhit_token_mint.decimals as u32); // 10 million tokens

        // Mint 4 million Solhit Tokens to the vesting_contract_account (this contract's account for managing rewards)
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.solhit_token_mint.to_account_info(),
                    to: ctx.accounts.vesting_contract_account.to_account_info(),
                    authority: ctx.accounts.admin.to_account_info(),
                },
                &[&[/* seeds used for the vesting_contract_account */]],
            ),
            rewards_amount,
        )?;

        // Mint the remaining 10 million Solhit Tokens to the specified distribution account
        // Ensure a TokenAccount exists for the exchange_or_other_destination and is passed here
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.solhit_token_mint.to_account_info(),
                    to: ctx.accounts.distribution_account.to_account_info(), // This should be the TokenAccount for exchange_or_other_destination
                    authority: ctx.accounts.admin.to_account_info(),
                },
            ),
            distribution_amount,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub buyer_token_account: Account<'info, TokenAccount>, // Buyer's PledgeToken account
    #[account(mut)]
    pub ico_state: Account<'info, ICOState>, // ICO state account
    #[account(mut)]
    pub user_state: Account<'info, UserState>, // User state for this buyer
    #[account(mut)]
    pub pledge_token_mint: Account<'info, Mint>, // PledgeToken mint account
    /// CHECK: This is safe to do because we are only reading the data
    pub clock: Sysvar<'info, Clock>, // For accessing the current blockchain timestamp
    pub token_program: Program<'info, Token>, // SPL Token program
}

impl<'info> BuyTokens<'info> {
    pub fn buy_tokens(ctx: Context<BuyTokens>, amount_sol: u64) -> Result<()> {
        let ico_state = &mut ctx.accounts.ico_state;
        let user_state = &mut ctx.accounts.user_state;

        // Determine the current phase and token price
        let current_phase = determine_current_phase(&ico_state, &ctx.accounts.clock);
        let token_price = ico_state.phase_details[current_phase as usize].token_price;

        // Calculate the number of tokens to mint
        let tokens_to_mint = amount_sol / token_price; // Ensure you handle decimals appropriately

        // Ensure the purchase doesn't exceed the total tokens allocated for sale
        if ico_state.total_tokens_sold + tokens_to_mint > ico_state.total_tokens_allocated {
            return Err(MyContractError::OverPurchase.into()); // Replace 0 with your error code for over-purchase
        }

        // Mint tokens to the buyer's account
        let cpi_accounts = MintTo {
            mint: ctx.accounts.pledge_token_mint.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.ico_state.to_account_info(), // Assuming ICO contract has minting authority
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, tokens_to_mint)?;

        // Update ICO and user states
        ico_state.total_tokens_sold += tokens_to_mint;
        user_state.amount_purchased += tokens_to_mint;
        // Optionally update vesting_start_timestamp if necessary

        Ok(())
    }
}

fn determine_current_phase(ico_state: &ICOState, clock: &Sysvar<Clock>) -> u8 {
    let now = clock.unix_timestamp;
    let mut elapsed_time = now - ico_state.start_timestamp;

    for (i, phase) in ico_state.phase_details.iter().enumerate() {
        if elapsed_time <= phase.duration {
            return i as u8;
        }
        elapsed_time -= phase.duration;
    }

    // If the time exceeds all phase durations, return the last phase by default
    // or handle as needed for your logic (e.g., ICO has ended)
    (ico_state.phase_details.len() - 1) as u8
}

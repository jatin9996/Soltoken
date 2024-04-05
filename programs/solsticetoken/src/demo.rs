use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};

declare_id!("Fg6PaFhzQxQfen8DHuZQDVpokndbCWTJcDJTYYkg4LTG");

#[program]
pub mod demo_ico {
    use super::*;

    pub fn start_ico(ctx: Context<StartIco>, phase_details: Vec<Phase>) -> Result<()> {
        let ico_state = &mut ctx.accounts.ico_state;
        ico_state.phases = phase_details;
        Ok(())
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount_sol: u64) -> Result<()> {
        let ico_state = &ctx.accounts.ico_state;

        if let Some(phase) = ico_state.current_phase() {
            let tokens_to_mint = amount_sol / phase.token_price; // Simplified calculation
                                                                 // Here you would include logic to mint tokens to the vesting contract
            msg!("Tokens to mint: {}", tokens_to_mint);
        } else {
            return Err(ErrorCode::OutsideICOPhase.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct StartIco<'info> {
    #[account(init, payer = admin, space = 9000)]
    pub ico_state: Account<'info, ICOState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub ico_state: Account<'info, ICOState>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct ICOState {
    pub phases: Vec<Phase>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Phase {
    pub token_price: u64,
    pub start: i64,
    pub end: i64,
}

impl ICOState {
    fn current_phase(&self) -> Option<&Phase> {
        let now = Clock::get().unwrap().unix_timestamp;
        self.phases
            .iter()
            .find(|phase| now >= phase.start && now <= phase.end)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("The transaction is outside the ICO phase.")]
    OutsideICOPhase,
}

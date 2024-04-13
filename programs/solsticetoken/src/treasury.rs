use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;

declare_id!("...");

#[program]
pub mod treasury {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let treasury_state = &mut ctx.accounts.treasury_state;
        treasury_state.admin = admin;
        Ok(())
    }

    pub fn add_beneficiary(ctx: Context<ManageBeneficiary>, beneficiary: Pubkey, amount: u64) -> Result<()> {
        require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.treasury_state.admin, ErrorCode::Unauthorized);

        let beneficiary_info = BeneficiaryInfo {
            beneficiary,
            amount,
        };
        ctx.accounts.treasury_state.beneficiaries.push(beneficiary_info);
        Ok(())
    }

    pub fn remove_beneficiary(ctx: Context<ManageBeneficiary>, beneficiary: Pubkey) -> Result<()> {
        require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.treasury_state.admin, ErrorCode::Unauthorized);

        let index = ctx.accounts.treasury_state.beneficiaries.iter().position(|x| x.beneficiary == beneficiary).ok_or(ErrorCode::BeneficiaryNotFound)?;
        ctx.accounts.treasury_state.beneficiaries.remove(index);
        Ok(())
    }

    pub fn claim_funds(ctx: Context<ClaimFunds>) -> Result<()> {
        let beneficiary_info = ctx.accounts.treasury_state.beneficiaries.iter().find(|x| x.beneficiary == *ctx.accounts.beneficiary.key()).ok_or(ErrorCode::Unauthorized)?;
        **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? -= beneficiary_info.amount;
        **ctx.accounts.beneficiary.to_account_info().try_borrow_mut_lamports()? += beneficiary_info.amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + (40 * 10))] // Assuming a max of 10 beneficiaries for demonstration
    pub treasury_state: Account<'info, TreasuryState>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageBeneficiary<'info> {
    #[account(mut)]
    pub treasury_state: Account<'info, TreasuryState>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimFunds<'info> {
    #[account(mut)]
    pub treasury: Account<'info, TreasuryState>,
    #[account(mut)]
    pub beneficiary: Signer<'info>,
}

#[account]
pub struct TreasuryState {
    pub admin: Pubkey,
    pub beneficiaries: Vec<BeneficiaryInfo>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BeneficiaryInfo {
    pub beneficiary: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized action.")]
    Unauthorized,
    #[msg("Beneficiary not found.")]
    BeneficiaryNotFound,
}

use anchor_lang::prelude::*;

#[account]
pub struct ICOState {
    pub start_timestamp: i64,
    pub current_phase: u8,
    pub total_tokens_sold: u64,
    pub phase_details: Vec<PhaseDetail>,
    pub total_tokens_allocated: u64,
}

#[account]
pub struct UserState {
    pub user: Pubkey,
    pub amount_purchased: u64,
    pub vesting_start_timestamp: i64,
    pub rewards_claimed: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PhaseDetail {
    pub phase: u8,
    pub duration: i64,
    pub token_price: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardEntitlement {
    pub amount: u64,
}

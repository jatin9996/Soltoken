
use anchor_lang::prelude::*;

#[account]
pub struct MLMParticipant {
    pub user: Pubkey,
    pub level: u8,
    pub rewards_earned: u64,
}

pub struct MLMStrategy;

impl MLMStrategy {
    pub fn initialize_participant(participant: AccountInfo, user: Pubkey) -> Result<()> {
        let mut participant_data = MLMParticipant::try_from_slice(&participant.data.borrow())?;
        participant_data.user = user;
        participant_data.level = 1; // Default level
        participant_data.rewards_earned = 0;
        participant.data.borrow_mut().copy_from_slice(&participant_data.try_to_vec()?);
        Ok(())
    }

    pub fn calculate_rewards(participant: AccountInfo) -> Result<u64> {
        let participant_data = MLMParticipant::try_from_slice(&participant.data.borrow())?;
        let rewards = participant_data.level as u64 * 100; 
        Ok(rewards)
    }
}

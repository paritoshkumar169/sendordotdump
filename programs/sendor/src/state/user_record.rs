use anchor_lang::prelude::*;

#[account]
pub struct UserRecord {
    pub user: Pubkey,        // The user's wallet address
    pub last_action_day: u64, // The last day index in which the user performed an action
}

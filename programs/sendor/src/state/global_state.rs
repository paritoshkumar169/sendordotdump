use anchor_lang::prelude::*;

#[account]
pub struct GlobalState {
    pub admin: Pubkey,        // Administrator of the program (allowed to initialize and manage launches)
    pub launch_count: u64,    // Counter for number of launches created (used for PDA seeds)
}

use anchor_lang::prelude::*;

#[account]
pub struct UserRecord {
    pub user: Pubkey,
    pub last_action_day: u64,
}

impl UserRecord {
    /// 8-byte discriminator + 32 (user) + 8 (day)
    pub const LEN: usize = 8 + 32 + 8;
}

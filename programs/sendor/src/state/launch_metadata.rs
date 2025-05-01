use anchor_lang::prelude::*;

#[account]
pub struct LaunchMetadata {
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub launch_id: u64,
    pub current_day: u64,
    pub window1_start: i64,
    pub window1_len: i64,
    pub window2_start: i64,
    pub window2_len: i64,
    pub bump: u8,
}

impl LaunchMetadata {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1;

    pub fn is_window_open(&self, now: i64) -> bool {
        let t = now % 86_400;
        (t >= self.window1_start && t < self.window1_start + self.window1_len)
            || (t >= self.window2_start && t < self.window2_start + self.window2_len)
    }
}

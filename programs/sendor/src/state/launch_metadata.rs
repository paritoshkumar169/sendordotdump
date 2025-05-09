use anchor_lang::prelude::*;

#[account]
pub struct LaunchMetadata {
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub launch_id: u64,
    pub current_day: u64,
    pub window1_start: i64,
    pub window2_start: i64,
    pub bump: u8,
    // Token metadata fields
    pub token_name: String,    // Max 32 bytes
    pub token_symbol: String,  // Max 10 bytes
    pub token_uri: String,     // Max 200 bytes
}

impl LaunchMetadata {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 1 + 32 + 10 + 200;

    pub fn is_window_open(&self, now: i64) -> bool {
        const LEN: i64 = 900;
        let t = now % 86_400;
        (t >= self.window1_start && t < self.window1_start + LEN)
            || (t >= self.window2_start && t < self.window2_start + LEN)
    }
}

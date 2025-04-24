use anchor_lang::prelude::*;

#[account]
pub struct LaunchMetadata {
    pub token_mint: Pubkey,     // The SPL token mint for this launch
    pub vault: Pubkey,          // Token account (vault) holding unsold tokens
    pub launch_id: u64,         // Unique launch identifier (matches the global launch_count at creation)
    pub current_day: u64,       // Current day index for trading cycles (increments daily)
    pub window1_start: i64,     // Start timestamp of the first daily trading window (UTC Unix time)
    pub window2_start: i64,     // Start timestamp of the second daily trading window (UTC Unix time)
    pub bump: u8,               // PDA bump for this LaunchMetadata (for signing authority)
}

use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState {
    /// Administrator of the launchpad (initial signer)
    pub admin:        Pubkey,
    /// Wallet to receive platform fees for token launches
    pub platform_fee_recipient: Pubkey,
    /// Fee amount in lamports for creating a new launch
    pub launch_fee_lamports: u64,
    /// Counts how many token launches have been created; each `create_launch`
    /// increments this and uses the value as a seed.
    pub launch_count: u64,
    /// PDA bump for `global_state` (handy for future CPI calls)
    pub bump:         u8,
    /// Padding to ensure proper alignment
    pub _padding: [u8; 7], // Add padding to ensure 8-byte alignment
}

impl GlobalState {
    /// Account size: 8-byte Anchor discriminator + 32 (admin) + 32 (fee_recipient) + 8 (fee_lamports) + 8 (count) + 1 (bump) + 7 (padding)
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 1 + 7;
}

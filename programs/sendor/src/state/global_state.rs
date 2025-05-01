use anchor_lang::prelude::*;

#[account]
pub struct GlobalState {
    /// Administrator of the launchpad (initial signer)
    pub admin:        Pubkey,

    /// Counts how many token launches have been created; each `create_launch`
    /// increments this and uses the value as a seed.
    pub launch_count: u64,

    /// PDA bump for `global_state` (handy for future CPI calls)
    pub bump:         u8,
}

impl GlobalState {
    /// Account size: 8-byte Anchor discriminator + 32 (admin) + 8 (count) + 1 (bump)
    pub const LEN: usize = 8 + 32 + 8 + 1;
}

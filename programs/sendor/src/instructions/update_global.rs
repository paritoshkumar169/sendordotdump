use anchor_lang::prelude::*;
use crate::errors::LaunchError;
use crate::state::global_state::GlobalState;
use crate::state::launch_metadata::LaunchMetadata;
use crate::constants::*;

/// Context for the `update_global` instruction.
#[derive(Accounts)]
pub struct UpdateGlobal<'info> {
    #[account(mut, has_one = admin)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    pub admin: Signer<'info>,
    pub clock: Sysvar<'info, Clock>,
}

/// Pseudo-randomly selects new daily trading windows and advances the trading day.
pub fn update_global(ctx: Context<UpdateGlobal>) -> Result<()> {
    let global = &mut ctx.accounts.global_state;
    let launch = &mut ctx.accounts.launch_metadata;
    require!(ctx.accounts.admin.key() == global.admin, LaunchError::Unauthorized);

    // Increment the global trading day for the launch
    launch.current_day = launch.current_day.checked_add(1).unwrap();

    // Derive randomness from on-chain clock (not cryptographically secure)
    let now = ctx.accounts.clock.unix_timestamp;
    let slot = Clock::get()?.slot;
    let rand1 = (now as u64) ^ slot;
    let rand2 = rand1.rotate_left(17) ^ (now as u64).rotate_right(13);

    // Random offsets within half-day window (ensuring windows do not overlap and roughly 12h apart)
    let half = HALF_DAY;
    let offset1 = (rand1 % ((half - WINDOW_DURATION) as u64)) as i64;
    let offset2 = (rand2 % ((half - WINDOW_DURATION) as u64)) as i64;
    // Set window start times relative to current time
    let base_time = now;  // treat now as the start of the new cycle
    let w1_start = base_time + offset1;
    let w2_start = base_time + half + offset2;
    // Ensure second window is after the first window
    require!(w2_start >= w1_start + WINDOW_DURATION, LaunchError::InvalidWindowTimes);

    launch.window1_start = w1_start;
    launch.window2_start = w2_start;
    Ok(())
}

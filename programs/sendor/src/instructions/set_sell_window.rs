use anchor_lang::prelude::*;
use crate::errors::LaunchError;
use crate::state::global_state::GlobalState;
use crate::state::launch_metadata::LaunchMetadata;
use crate::constants::*;

#[derive(Accounts)]
pub struct SetSellWindow<'info> {
    #[account(has_one = admin)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    pub admin: Signer<'info>,
}

pub fn set_sell_window(ctx: Context<SetSellWindow>, window1_start: i64, window2_start: i64) -> Result<()> {
    let global = &ctx.accounts.global_state;
    let launch = &mut ctx.accounts.launch_metadata;
    require!(ctx.accounts.admin.key() == global.admin, LaunchError::Unauthorized);
    require!(window2_start >= window1_start + WINDOW_DURATION, LaunchError::InvalidWindowTimes);

    // Advance the day counter and set specified window times
    launch.current_day = launch.current_day.checked_add(1).unwrap();
    launch.window1_start = window1_start;
    launch.window2_start = window2_start;
    Ok(())
}

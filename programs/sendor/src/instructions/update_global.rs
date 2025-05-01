use anchor_lang::prelude::*;
use crate::errors::LaunchError;
use crate::state::{global_state::GlobalState, launch_metadata::LaunchMetadata};

const DAY: i64      = 86_400;
const WIN: i64      = 900;
const MIN_GAP: i64  = 43_200;  // 12 h
const MAX_GAP: i64  = 64_800;  // 18 h

#[event]
pub struct GlobalUpdated {
    pub launch_id: u64,
    pub day:       u64,
    pub w1:        i64,
    pub w2:        i64,
}

#[derive(Accounts)]
pub struct UpdateGlobal<'info> {
    #[account(has_one = admin)]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub launch_metadata: Account<'info, LaunchMetadata>,

    pub admin: Signer<'info>,
}

pub fn update_global(ctx: Context<UpdateGlobal>) -> Result<()> {
    let launch = &mut ctx.accounts.launch_metadata;

    launch.current_day = launch
        .current_day
        .checked_add(1)
        .ok_or(LaunchError::MathOverflow)?;

    let seed  = Clock::get()?.slot.wrapping_add(launch.launch_id);
    let w1    = (seed % ((DAY - MAX_GAP - WIN) as u64)) as i64;
    let gap   = MIN_GAP + ((seed >> 8) % ((MAX_GAP - MIN_GAP) as u64)) as i64;
    let w2    = w1 + gap;

    require!(w2 + WIN <= DAY, LaunchError::InvalidWindowTimes);

    launch.window1_start = w1;
    launch.window2_start = w2;

    emit!(GlobalUpdated {
        launch_id: launch.launch_id,
        day:       launch.current_day,
        w1,
        w2,
    });

    Ok(())
}

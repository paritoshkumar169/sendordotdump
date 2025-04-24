use anchor_lang::prelude::*;
use crate::state::global_state::GlobalState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 8, seeds = [b"global"], bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let global = &mut ctx.accounts.global_state;
    global.admin = ctx.accounts.admin.key();
    global.launch_count = 0;
    Ok(())
}

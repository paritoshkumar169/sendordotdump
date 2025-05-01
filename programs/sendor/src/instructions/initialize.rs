use anchor_lang::prelude::*;
use crate::state::global_state::GlobalState;

#[event]
pub struct Initialization {
    pub admin: Pubkey,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // One-time PDA; a second call fails because the account
    // already exists at this deterministic seed.
    #[account(
        init,
        payer  = admin,
        space  = GlobalState::LEN,
        seeds  = [b"global"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent:           Sysvar<'info, Rent>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let global = &mut ctx.accounts.global_state;

    global.admin         = ctx.accounts.admin.key();
    global.launch_count  = 0;
    global.bump          = ctx.bumps.global_state;   // <-- fixed line

    emit!(Initialization { admin: global.admin });
    Ok(())
}

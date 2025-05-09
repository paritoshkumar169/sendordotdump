use anchor_lang::prelude::*;
use crate::state::global_state::GlobalState;

#[event]
pub struct Initialization {
    pub admin: Pubkey,
    pub platform_fee_recipient: Pubkey,
    pub launch_fee_lamports: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    // One-time PDA; a second call fails because the account
    // already exists at this deterministic seed.
    #[account(
        init,
        payer  = admin,
        space  = 8 + std::mem::size_of::<GlobalState>(), // 8-byte discriminator + struct size
        seeds = [b"global_v2"], 
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent:           Sysvar<'info, Rent>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    platform_fee_recipient: Pubkey,
    launch_fee_lamports: u64,
) -> Result<()> {
    let global = &mut ctx.accounts.global_state;

    global.admin         = ctx.accounts.admin.key();
    global.platform_fee_recipient = platform_fee_recipient;
    global.launch_fee_lamports = launch_fee_lamports;
    global.launch_count  = 0;
    global.bump          = ctx.bumps.global_state;

    emit!(Initialization {
        admin: global.admin,
        platform_fee_recipient: global.platform_fee_recipient,
        launch_fee_lamports: global.launch_fee_lamports,
    });
    Ok(())
}

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::launch_metadata::LaunchMetadata;
use crate::state::user_record::UserRecord;


#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut, has_one = token_mint, has_one = vault)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = source_token_account.owner == source.key() && source_token_account.mint == launch_metadata.token_mint)]
    pub source_token_account: Account<'info, TokenAccount>,

    pub destination: SystemAccount<'info>,
    #[account(init_if_needed, payer = source, associated_token::mint = token_mint, associated_token::authority = destination)]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(init_if_needed, payer = source,
        seeds = [b"user_record", launch_metadata.key().as_ref(), source.key().as_ref()],
        bump,
        space = 8 + 32 + 8)]
    pub user_record: Account<'info, UserRecord>,
    #[account(mut)]
    pub source: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Allows a user to transfer tokens to another wallet within the constraints of the daily transfer window and limit.
pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
    let launch = &ctx.accounts.launch_metadata;
    let user_record = &mut ctx.accounts.user_record;
    let source = &ctx.accounts.source;
    let source_account = &ctx.accounts.source_token_account;
    let dest_account = &ctx.accounts.destination_token_account;

    // Enforce that current time is within a transfer window
    let now_ts = Clock::get()?.unix_timestamp;
    let w1 = launch.window1_start;
    let w2 = launch.window2_start;
    let open = (now_ts >= w1 && now_ts < w1 + WINDOW_DURATION) ||
               (now_ts >= w2 && now_ts < w2 + WINDOW_DURATION);
    require!(open, LaunchError::NotInTradingWindow);

    // One action per day per user
    if user_record.last_action_day != 0 {
        require!(user_record.last_action_day < launch.current_day, LaunchError::ActionAlreadyPerformed);
    }
    // Enforce 20% transfer limit
    let balance = source_account.amount;
    let max_transfer = balance / 5;
    require!(amount <= max_transfer, LaunchError::ExceedsTransferLimit);
    require!(amount > 0, LaunchError::ExceedsTransferLimit);

    // Transfer tokens from source to destination
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: source_account.to_account_info(),
                to: dest_account.to_account_info(),
                authority: source.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update user record
    user_record.user = source.key();
    user_record.last_action_day = launch.current_day;
    Ok(())
}

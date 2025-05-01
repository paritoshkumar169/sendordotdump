use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::errors::LaunchError;
use crate::state::{launch_metadata::LaunchMetadata, user_record::UserRecord};

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub qty: u64,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(has_one = token_mint)]
    pub launch_metadata: Account<'info, LaunchMetadata>,

    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(mut)]
    pub from: Signer<'info>,

    #[account(
        mut,
        constraint = source_token_account.owner == from.key(),
        constraint = source_token_account.mint == token_mint.key()
    )]
    pub source_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = from,
        associated_token::mint = token_mint,
        associated_token::authority = to
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    /// CHECK: arbitrary receiver
    pub to: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = from,
        space = UserRecord::LEN,
        seeds = [b"user", launch_metadata.key().as_ref(), from.key().as_ref()],
        bump
    )]
    pub user_record: Account<'info, UserRecord>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
    let launch = &ctx.accounts.launch_metadata;
    let record = &mut ctx.accounts.user_record;
    let from_account = &ctx.accounts.source_token_account;

    let now = Clock::get()?.unix_timestamp;
    require!(launch.is_window_open(now), LaunchError::NotInTradingWindow);

    let today = (now / 86_400) as u64;
    require!(record.last_action_day != today, LaunchError::ActionAlreadyPerformed);

    let max_transfer = from_account.amount / 5;
    require!(amount <= max_transfer, LaunchError::ExceedsTransferLimit);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: from_account.to_account_info(),
                to: ctx.accounts.destination_token_account.to_account_info(),
                authority: ctx.accounts.from.to_account_info(),
            },
        ),
        amount,
    )?;

    record.last_action_day = today;
    if record.user == Pubkey::default() {
        record.user = ctx.accounts.from.key();
    }

    emit!(TransferEvent {
        from: ctx.accounts.from.key(),
        to: ctx.accounts.to.key(),
        qty: amount,
    });

    Ok(())
}

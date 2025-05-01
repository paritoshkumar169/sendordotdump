use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, TokenAccount, TransferChecked};
use crate::errors::LaunchError;
use crate::state::{
    bonding_curve_state::BondingCurveState,
    launch_metadata::LaunchMetadata,
    user_record::UserRecord,
};

#[event]
pub struct SellEvent {
    pub seller: Pubkey,
    pub qty: u64,
    pub payout: u64,
}

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut, has_one = vault, has_one = token_mint)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    #[account(mut, has_one = launch_metadata)]
    pub bonding_curve: Account<'info, BondingCurveState>,
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        mut,
        constraint = seller_token_account.owner == seller.key(),
        constraint = seller_token_account.mint == token_mint.key()
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer  = seller,
        space  = UserRecord::LEN,
        seeds  = [b"user", launch_metadata.key().as_ref(), seller.key().as_ref()],
        bump
    )]
    pub user_record: Account<'info, UserRecord>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn sell(ctx: Context<Sell>, amount: u64, min_payout: u64) -> Result<()> {
    let launch = &mut ctx.accounts.launch_metadata;
    let curve = &mut ctx.accounts.bonding_curve;
    let seller = &ctx.accounts.seller;
    let seller_account = &ctx.accounts.seller_token_account;
    let record = &mut ctx.accounts.user_record;

    require!(curve.decimals <= 18, LaunchError::InvalidDecimals);

    let now = Clock::get()?.unix_timestamp;
    require!(launch.is_window_open(now), LaunchError::NotInTradingWindow);

    let today = (now / 86_400) as u64;
    require!(record.last_action_day != today, LaunchError::ActionAlreadyPerformed);

    let balance = seller_account.amount;
    let max_sell = balance.checked_mul(10).ok_or(LaunchError::MathOverflow)? / 100;
    require!(amount <= max_sell, LaunchError::ExceedsSellLimit);

    let payout = curve.compute_payout(amount)?;
    require!(payout >= min_payout, LaunchError::PayoutTooLow);

    let launch_lamports = **launch.to_account_info().lamports.borrow();
    require!(payout <= launch_lamports, LaunchError::InsufficientLiquidity);

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: seller_account.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: seller.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
            },
        ),
        amount,
        curve.decimals,
    )?;

    let id_bytes = launch.launch_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"launch", id_bytes.as_ref(), &[launch.bump]];
    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: launch.to_account_info(),
                to: seller.to_account_info(),
            },
            &[seeds],
        ),
        payout,
    )?;

    curve.current_supply = curve
        .current_supply
        .checked_sub(amount)
        .ok_or(LaunchError::MathOverflow)?;
    record.last_action_day = today;
    if record.user == Pubkey::default() {
        record.user = seller.key();
    }

    emit!(SellEvent {
        seller: seller.key(),
        qty: amount,
        payout,
    });
    Ok(())
}

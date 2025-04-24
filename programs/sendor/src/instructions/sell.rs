use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::launch_metadata::LaunchMetadata;
use crate::state::bonding_curve_state::BondingCurveState;
use crate::state::user_record::UserRecord;

/// Context for the `sell` instruction.
#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut, has_one = token_mint, has_one = vault)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    #[account(mut, has_one = launch_metadata)]
    pub bonding_curve: Account<'info, BondingCurveState>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut, constraint = user_token_account.owner == seller.key() && user_token_account.mint == launch_metadata.token_mint)]
    pub user_token_account: Account<'info, TokenAccount>,  // seller's token source
    #[account(init_if_needed, payer = seller,
        seeds = [b"user_record", launch_metadata.key().as_ref(), seller.key().as_ref()],
        bump,
        space = 8 + 32 + 8)]
    pub user_record: Account<'info, UserRecord>,
    #[account(mut)]
    pub seller: Signer<'info>,  // user selling tokens
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Allows a user to sell tokens back to the launchpad (receive lamports according to bonding curve).
pub fn sell(ctx: Context<Sell>, amount: u64) -> Result<()> {
    let launch = &mut ctx.accounts.launch_metadata;
    let curve = &mut ctx.accounts.bonding_curve;
    let seller = &mut ctx.accounts.seller;
    let user_record = &mut ctx.accounts.user_record;
    let user_token_account = &ctx.accounts.user_token_account;
    let vault = &mut ctx.accounts.vault;

    // Enforce that the current time is within a sell window
    let now_ts = Clock::get()?.unix_timestamp;
    let w1 = launch.window1_start;
    let w2 = launch.window2_start;
    let open = (now_ts >= w1 && now_ts < w1 + WINDOW_DURATION) ||
               (now_ts >= w2 && now_ts < w2 + WINDOW_DURATION);
    require!(open, LaunchError::NotInTradingWindow);

    // Enforce one action per day per user
    if user_record.last_action_day != 0 {
        require!(user_record.last_action_day < launch.current_day, LaunchError::ActionAlreadyPerformed);
    }
    // Compute 10% of user's holdings
    let balance = user_token_account.amount;
    let max_sell = balance / 10;
    require!(amount <= max_sell, LaunchError::ExceedsSellLimit);
    require!(amount > 0, LaunchError::ExceedsSellLimit);

    // Ensure pool has enough lamports for payout
    let decimals = curve.decimals;
    let base_price = curve.base_price as u128;
    let slope = curve.slope as u128;
    let supply_before = curve.current_supply as u128;
    let sell_amount = amount as u128;
    // Payout = base_price * sell_amount + slope * (supply_before * sell_amount - sell_amount^2/2) / m
    let m = 10u128.pow(decimals as u32);
    let numerator = base_price * sell_amount * m 
        + slope * (supply_before * sell_amount - sell_amount * (sell_amount - 1) / 2);
    let payout = (numerator / (m * m)) as u64;
    let pool_balance = **launch.to_account_info().lamports.borrow();
    require!(payout <= pool_balance, LaunchError::InsufficientLiquidity);

    // Transfer tokens from seller to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: user_token_account.to_account_info(),
                to: vault.to_account_info(),
                authority: seller.to_account_info(),
            },
        ),
        amount,
    )?;
    // Transfer lamports from launch pool to seller
    **launch.to_account_info().try_borrow_mut_lamports()? -= payout;
    **seller.to_account_info().try_borrow_mut_lamports()? += payout;

    // Update bonding curve supply
    curve.current_supply = curve.current_supply.checked_sub(amount).unwrap();
    // Update user record for daily action
    user_record.user = seller.key();
    user_record.last_action_day = launch.current_day;
    Ok(())
}

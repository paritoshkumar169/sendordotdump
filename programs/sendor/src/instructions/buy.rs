use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, TransferChecked};

use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::{bonding_curve_state::BondingCurveState, launch_metadata::LaunchMetadata};

const DAY: i64 = 86_400;
const WIN: i64 = 900;
const MIN_GAP: i64 = 43_200;
const MAX_GAP: i64 = 64_800;

#[event]
pub struct PurchaseEvent {
    pub buyer: Pubkey,
    pub qty:   u64,
    pub cost:  u64,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut, has_one = token_mint, has_one = vault)]
    pub launch_metadata: Account<'info, LaunchMetadata>,

    #[account(mut, has_one = launch_metadata)]
    pub bonding_curve: Account<'info, BondingCurveState>,

    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub vault:      Account<'info, TokenAccount>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint      = token_mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:           Program<'info, System>,
    pub rent:                     Sysvar<'info, Rent>,
}

pub fn buy(ctx: Context<Buy>, amount: u64, max_cost: u64) -> Result<()> {
    let launch  = &mut ctx.accounts.launch_metadata;
    let curve   = &mut ctx.accounts.bonding_curve;
    let vault   = &mut ctx.accounts.vault;
    let buyer   = &ctx.accounts.buyer;
    let buyer_token_account = &ctx.accounts.buyer_token_account;

    let now = Clock::get()?.unix_timestamp;
    let today = (now / DAY) as u64;

    if today > launch.current_day {
        launch.current_day = today;
        let seed = Clock::get()?.slot.wrapping_add(launch.launch_id);
        let w1   = (seed % ((DAY - MAX_GAP - WIN) as u64)) as i64;
        let gap  = MIN_GAP + ((seed >> 8) % ((MAX_GAP - MIN_GAP) as u64)) as i64;
        let w2   = w1 + gap;
        require!(w2 + WIN <= DAY, LaunchError::InvalidWindowTimes);
        launch.window1_start = w1;
        launch.window2_start = w2;
    }

    require!(curve.decimals <= 18, LaunchError::InvalidDecimals);

    let available = INITIAL_SUPPLY_BASE_UNITS
        .checked_sub(curve.current_supply)
        .ok_or(LaunchError::InsufficientSupply)?;
    require!(amount <= available, LaunchError::InsufficientSupply);

    let cost = compute_cost(curve, amount)?;
    require!(cost <= max_cost, LaunchError::SlippageExceeded);

    let buyer_lamports = **buyer.to_account_info().lamports.borrow();
    require!(cost <= buyer_lamports, LaunchError::InsufficientFunds);

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: buyer.to_account_info(),
                to:   launch.to_account_info(),
            },
        ),
        cost,
    )?;

    let id_bytes = launch.launch_id.to_le_bytes();
    let seeds: &[&[u8]] = &[b"launch", id_bytes.as_ref(), &[launch.bump]];

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      vault.to_account_info(),
                to:        buyer_token_account.to_account_info(),
                authority: launch.to_account_info(),
                mint:      ctx.accounts.token_mint.to_account_info(),
            },
            &[seeds],
        ),
        amount,
        curve.decimals,
    )?;

    curve.current_supply = curve
        .current_supply
        .checked_add(amount)
        .ok_or(LaunchError::MathOverflow)?;

    emit!(PurchaseEvent {
        buyer: buyer.key(),
        qty:   amount,
        cost,
    });

    Ok(())
}

fn compute_cost(curve: &BondingCurveState, qty: u64) -> Result<u64> {
    let m       = 10u128.pow(curve.decimals as u32);
    let base    = curve.base_price     as u128;
    let slope   = curve.slope          as u128;
    let supply  = curve.current_supply as u128;
    let qty128  = qty                  as u128;

    let part1 = base
        .checked_mul(qty128)
        .ok_or(LaunchError::MathOverflow)?
        .checked_mul(m)
        .ok_or(LaunchError::MathOverflow)?;

    let part2 = slope
        .checked_mul(
            supply
                .checked_mul(qty128)
                .ok_or(LaunchError::MathOverflow)?
                .checked_add(qty128 * (qty128 + 1) / 2)
                .ok_or(LaunchError::MathOverflow)?,
        )
        .ok_or(LaunchError::MathOverflow)?;

    let numerator = part1.checked_add(part2).ok_or(LaunchError::MathOverflow)?;
    let denom     = m.checked_mul(m).ok_or(LaunchError::MathOverflow)?;

    Ok((numerator / denom) as u64)
}

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::launch_metadata::LaunchMetadata;
use crate::state::bonding_curve_state::BondingCurveState;


#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut, has_one = token_mint, has_one = vault)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    #[account(mut, has_one = launch_metadata)]
    pub bonding_curve: Account<'info, BondingCurveState>,
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,  // token vault holding remaining tokens
    #[account(mut)]
    pub buyer: Signer<'info>,                // buyer's wallet (pays lamports)
    #[account(init_if_needed, payer = buyer, associated_token::mint = token_mint, associated_token::authority = buyer)]
    pub buyer_token_account: Account<'info, TokenAccount>,  // buyer's token account to receive tokens
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
    let launch = &mut ctx.accounts.launch_metadata;
    let curve  = &mut ctx.accounts.bonding_curve;
    let vault  = &mut ctx.accounts.vault;
    let buyer_token_account = &mut ctx.accounts.buyer_token_account;
    let buyer  = &mut ctx.accounts.buyer;


    let available = INITIAL_SUPPLY_BASE_UNITS.checked_sub(curve.current_supply).unwrap();
    require!(amount <= available, LaunchError::InsufficientSupply);


    let decimals = curve.decimals;
    let base_price = curve.base_price as u128;
    let slope = curve.slope as u128;
    let supply_before = curve.current_supply as u128;
    let buy_amount = amount as u128;
    let m = 10u128.pow(decimals as u32);

    let numerator = base_price * buy_amount * m 
        + slope * (supply_before * buy_amount + buy_amount * (buy_amount + 1) / 2);
    let cost = (numerator / (m * m)) as u64;  // integer division floors the result

    let buyer_lamports = **buyer.to_account_info().lamports.borrow();
    require!(cost <= buyer_lamports, LaunchError::InsufficientFunds);


    **buyer.to_account_info().try_borrow_mut_lamports()? -= cost;
    **launch.to_account_info().try_borrow_mut_lamports()? += cost;


    let id_bytes = launch.launch_id.to_le_bytes();
    let seeds    = &[b"launch", id_bytes.as_ref(), &[launch.bump]];
    let signer_seeds = &[&seeds[..]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: vault.to_account_info(),
                to: buyer_token_account.to_account_info(),
                authority: launch.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;


    curve.current_supply = curve.current_supply.checked_add(amount).unwrap();
    Ok(())
}

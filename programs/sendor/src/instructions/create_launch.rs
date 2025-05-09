use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, SetAuthority};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_lang::system_program; // Required for SystemProgram.transfer

use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::{
    global_state::GlobalState,
    launch_metadata::LaunchMetadata,
    bonding_curve_state::BondingCurveState,
};

const MIN_BASE_PRICE_LAMPORTS: u64 = 1;
// const MAX_FINAL_PRICE_LAMPORTS: u64 = 100_000_000_000; // 100 SOL in lamports, adjust as needed
const MAX_FINAL_PRICE_LAMPORTS: u64 = 100 * 1_000_000_000; // Example: 100 SOL


#[event]
pub struct LaunchCreated {
    pub id: u64,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub token_name: String,
    pub token_symbol: String,
    pub token_uri: String,
}

#[derive(Accounts)]
#[instruction(token_name: String, token_symbol: String, token_uri: String)]
pub struct CreateLaunch<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer = creator,
        space = LaunchMetadata::LEN,
        seeds = [b"launch", global_state.launch_count.to_le_bytes().as_ref()],
        bump
    )]
    pub launch_metadata: Account<'info, LaunchMetadata>,

    #[account(
        init,
        payer = creator,
        space = BondingCurveState::LEN,
        seeds = [b"bonding", global_state.launch_count.to_le_bytes().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurveState>,

    #[account(
        init,
        payer = creator,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = launch_metadata.key(),
        mint::freeze_authority = launch_metadata.key(),
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = token_mint,
        associated_token::authority = launch_metadata
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(mut)]
    pub platform_fee_recipient: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_launch(
    ctx: Context<CreateLaunch>,
    base_price: u64,
    slope: u64,
    token_name: String,
    token_symbol: String,
    token_uri: String,
) -> Result<()> {
    // Parameter Validations
    require!(TOKEN_DECIMALS <= 18, LaunchError::InvalidDecimals);
    require!(base_price >= MIN_BASE_PRICE_LAMPORTS, LaunchError::InvalidParams);
    require!(slope > 0, LaunchError::InvalidParams);
    require!(!token_name.is_empty() && token_name.len() <= 32, LaunchError::InvalidParams);
    require!(!token_symbol.is_empty() && token_symbol.len() <= 10, LaunchError::InvalidParams);
    require!(!token_uri.is_empty() && token_uri.len() <= 200, LaunchError::InvalidParams);

    let global = &mut ctx.accounts.global_state;
    let launch = &mut ctx.accounts.launch_metadata;
    let curve = &mut ctx.accounts.bonding_curve;
    let mint_account = &ctx.accounts.token_mint;
    let vault_account = &ctx.accounts.vault;
    let creator_account = &ctx.accounts.creator;

    // Check max price
    let max_price = base_price
        .checked_add(
            slope
                .checked_mul(INITIAL_SUPPLY_BASE_UNITS)
                .ok_or(LaunchError::MathOverflow)?,
        )
        .ok_or(LaunchError::MathOverflow)?;
    require!(max_price <= MAX_FINAL_PRICE_LAMPORTS, LaunchError::InvalidParams);

    // 1. Platform Fee Payment
    if global.launch_fee_lamports > 0 {
        require_keys_eq!(ctx.accounts.platform_fee_recipient.key(), global.platform_fee_recipient, LaunchError::InvalidFeeRecipient);
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: creator_account.to_account_info(),
                to: ctx.accounts.platform_fee_recipient.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, global.launch_fee_lamports)?;
    }

    // 2. Initialize LaunchMetadata & BondingCurveState
    let launch_id_bytes = global.launch_count.to_le_bytes();
    let (_pda_key, launch_bump) = Pubkey::find_program_address(&[b"launch", &launch_id_bytes], ctx.program_id);

    launch.token_mint = mint_account.key();
    launch.vault = vault_account.key();
    launch.launch_id = global.launch_count;
    launch.current_day = 0;
    launch.window1_start = 0;
    launch.window2_start = 0;
    launch.bump = launch_bump;
    // Store token metadata
    launch.token_name = token_name.clone();
    launch.token_symbol = token_symbol.clone();
    launch.token_uri = token_uri.clone();

    curve.launch_metadata = launch.key();
    curve.base_price = base_price;
    curve.slope = slope;
    curve.current_supply = 0;
    curve.decimals = TOKEN_DECIMALS;

    // 3. Mint Initial Supply to Vault
    let launch_seeds: &[&[u8]] = &[b"launch", launch_id_bytes.as_ref(), &[launch.bump]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: mint_account.to_account_info(),
                to: vault_account.to_account_info(),
                authority: launch.to_account_info(),
            },
            &[launch_seeds],
        ),
        INITIAL_SUPPLY_BASE_UNITS,
    )?;

    // Emit event
    emit!(LaunchCreated {
        id: launch.launch_id,
        creator: creator_account.key(),
        mint: mint_account.key(),
        vault: vault_account.key(),
        token_name,
        token_symbol,
        token_uri,
    });

    Ok(())
}

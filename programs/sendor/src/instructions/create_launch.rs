use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, SetAuthority};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::spl_token::instruction::AuthorityType;

use crate::constants::*;
use crate::errors::LaunchError;
use crate::state::{
    global_state::GlobalState,
    launch_metadata::LaunchMetadata,
    bonding_curve_state::BondingCurveState,
};

const MIN_BASE_PRICE_LAMPORTS: u64 = 1;   
const MAX_FINAL_PRICE_LAMPORTS: u64 = 100; 

#[event]
pub struct LaunchCreated {
    pub id:    u64,
    pub mint:  Pubkey,
    pub vault: Pubkey,
}

#[derive(Accounts)]
pub struct CreateLaunch<'info> {
    #[account(mut, has_one = admin)]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        init,
        payer  = admin,
        space  = LaunchMetadata::LEN,
        seeds  = [b"launch", global_state.launch_count.to_le_bytes().as_ref()],
        bump
    )]
    pub launch_metadata: Account<'info, LaunchMetadata>,

    #[account(
        init,
        payer  = admin,
        space  = BondingCurveState::LEN,
        seeds  = [b"bonding", global_state.launch_count.to_le_bytes().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurveState>,

    #[account(
        init,
        payer              = admin,
        mint::decimals     = TOKEN_DECIMALS,
        mint::authority    = launch_metadata.key(),
        mint::freeze_authority = launch_metadata.key(),
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer  = admin,
        associated_token::mint      = token_mint,
        associated_token::authority = launch_metadata
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program:           Program<'info, System>,
    pub token_program:            Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent:                     Sysvar<'info, Rent>,
}

pub fn create_launch(
    ctx: Context<CreateLaunch>,
    base_price: u64,
    slope: u64,
) -> Result<()> {

    require!(TOKEN_DECIMALS <= 18, LaunchError::InvalidDecimals);
    require!(base_price >= MIN_BASE_PRICE_LAMPORTS, LaunchError::InvalidParams);
    require!(slope > 0, LaunchError::InvalidParams);

    let max_price = base_price
        .checked_add(
            slope
                .checked_mul(INITIAL_SUPPLY_BASE_UNITS)
                .ok_or(LaunchError::MathOverflow)?,
        )
        .ok_or(LaunchError::MathOverflow)?;
    require!(max_price <= MAX_FINAL_PRICE_LAMPORTS, LaunchError::InvalidParams);

    let global  = &mut ctx.accounts.global_state;
    let launch  = &mut ctx.accounts.launch_metadata;
    let curve   = &mut ctx.accounts.bonding_curve;
    let mint    = &ctx.accounts.token_mint;
    let vault   = &ctx.accounts.vault;

    let launch_id_bytes = global.launch_count.to_le_bytes();
    let (_pda, bump) =
        Pubkey::find_program_address(&[b"launch", &launch_id_bytes], ctx.program_id);

    launch.token_mint    = mint.key();
    launch.vault         = vault.key();
    launch.launch_id     = global.launch_count;
    launch.current_day   = 0;
    launch.window1_start = 0;
    launch.window2_start = 0;
    launch.bump          = bump;

    curve.launch_metadata = launch.key();
    curve.base_price      = base_price;
    curve.slope           = slope;
    curve.current_supply  = 0;
    curve.decimals        = TOKEN_DECIMALS;

    let seeds: &[&[u8]] = &[b"launch", launch_id_bytes.as_ref(), &[bump]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint:      mint.to_account_info(),
                to:        vault.to_account_info(),
                authority: launch.to_account_info(),
            },
            &[seeds],
        ),
        INITIAL_SUPPLY_BASE_UNITS,
    )?;

    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint:   mint.to_account_info(),
                current_authority: launch.to_account_info(),
            },
            &[seeds],
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    global.launch_count = global
        .launch_count
        .checked_add(1)
        .ok_or(LaunchError::MathOverflow)?;

    emit!(LaunchCreated {
        id:    launch.launch_id,
        mint:  mint.key(),
        vault: vault.key(),
    });

    Ok(())
}

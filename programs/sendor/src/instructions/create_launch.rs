use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, SetAuthority};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use crate::constants::*;
use crate::state::global_state::GlobalState;
use crate::state::launch_metadata::LaunchMetadata;
use crate::state::bonding_curve_state::BondingCurveState;

#[derive(Accounts)]
pub struct CreateLaunch<'info> {
    #[account(mut, has_one = admin)]
    pub global_state: Account<'info, GlobalState>,
    
    #[account(init,
        payer = admin,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1,
        seeds = [b"launch", global_state.launch_count.to_le_bytes().as_ref()],
        bump)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    
    #[account(init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 8 + 1,
        seeds = [b"bonding", global_state.launch_count.to_le_bytes().as_ref()],
        bump)]
    pub bonding_curve: Account<'info, BondingCurveState>,
    
    #[account(init,
        payer = admin,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = launch_metadata.key(),
        mint::freeze_authority = launch_metadata.key())]
    pub token_mint: Account<'info, Mint>,
    
    #[account(init,
        payer = admin,
        associated_token::mint = token_mint,
        associated_token::authority = launch_metadata)]
    pub vault: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_launch(ctx: Context<CreateLaunch>, base_price: u64, slope: u64) -> Result<()> {
    let global = &mut ctx.accounts.global_state;
    let launch = &mut ctx.accounts.launch_metadata;
    let curve = &mut ctx.accounts.bonding_curve;
    let mint = &ctx.accounts.token_mint;
    let vault = &ctx.accounts.vault;

    launch.token_mint = mint.key();
    launch.vault = vault.key();
    launch.launch_id = global.launch_count;
    launch.current_day = 0;
    launch.window1_start = 0;
    launch.window2_start = 0;
    
    let (_pda, bump) = Pubkey::find_program_address(
        &[b"launch", global.launch_count.to_le_bytes().as_ref()],
        ctx.program_id
    );
    launch.bump = bump;

    curve.launch_metadata = launch.key();
    curve.base_price = base_price;
    curve.slope = slope;
    curve.current_supply = 0;
    curve.decimals = TOKEN_DECIMALS;

    let launch_count_bytes = global.launch_count.to_le_bytes();
    let seeds = &[b"launch", launch_count_bytes.as_ref(), &[launch.bump]];
    let signer = &[&seeds[..]];
    let total_supply: u64 = INITIAL_SUPPLY_BASE_UNITS;
    
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: vault.to_account_info(),
                authority: launch.to_account_info(),
            },
            signer,
        ),
        total_supply,
    )?;

    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: mint.to_account_info(),
                current_authority: launch.to_account_info(),
            },
            signer,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    global.launch_count = global.launch_count.checked_add(1).unwrap();
    Ok(())
}

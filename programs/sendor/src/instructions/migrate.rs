use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount, SetAuthority};
use anchor_spl::associated_token::AssociatedToken;
use spl_token::instruction::AuthorityType;
use crate::state::global_state::GlobalState;
use crate::state::launch_metadata::LaunchMetadata;
use crate::state::bonding_curve_state::BondingCurveState;


#[derive(Accounts)]
pub struct Migrate<'info> {
    #[account(mut, has_one = admin)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut, close = admin, has_one = token_mint, has_one = vault)]
    pub launch_metadata: Account<'info, LaunchMetadata>,
    #[account(mut, close = admin, has_one = launch_metadata)]
    pub bonding_curve: Account<'info, BondingCurveState>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, anchor_spl::token::Mint>,
    #[account(init_if_needed, payer = admin, associated_token::mint = token_mint, associated_token::authority = admin)]
    pub admin_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Migrates or finalizes a token launch: transfers remaining tokens and funds to admin and closes accounts.
pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
    let launch = &ctx.accounts.launch_metadata;
    let vault = &ctx.accounts.vault;
    let token_mint = &ctx.accounts.token_mint;
    let admin_token_account = &ctx.accounts.admin_token_account;

    // Only admin (checked by has_one) can perform migration
    // Transfer all remaining tokens from vault to admin's token account
    let remaining_tokens = vault.amount;
    if remaining_tokens > 0 {
        let seeds = &[b"launch", launch.launch_id.to_le_bytes().as_ref(), &[launch.bump]];
        let signer_seeds = &[&seeds[..]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: vault.to_account_info(),
                    to: admin_token_account.to_account_info(),
                    authority: launch.to_account_info(),
                },
                signer_seeds,
            ),
            remaining_tokens,
        )?;
    }

    // Revoke freeze authority on the mint (set to None) to fully decentralize the token
    let seeds = &[b"launch", launch.launch_id.to_le_bytes().as_ref(), &[launch.bump]];
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: token_mint.to_account_info(),
                current_authority: launch.to_account_info(),
            },
            &[&seeds[..]],
        ),
        AuthorityType::FreezeAccount,
        None,
    )?;

    // Close the vault token account (returns rent to admin)
    token::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: vault.to_account_info(),
                destination: ctx.accounts.admin.to_account_info(),
                authority: launch.to_account_info(),
            },
            &[&seeds[..]],
        ),
    )?;
    // The LaunchMetadata and BondingCurveState accounts will be closed automatically (close = admin)
    Ok(())
}

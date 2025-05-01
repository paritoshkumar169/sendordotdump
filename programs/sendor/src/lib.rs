use anchor_lang::prelude::*;

declare_id!("6mqsEaGREVXfAroU9WErmEPqYmKoFpoMHuFHzvBBGgna");

pub mod constants;
pub mod errors;
pub mod state;
pub mod instructions;

use crate::instructions::initialize::*;
use crate::instructions::create_launch::*;
use crate::instructions::buy::*;
use crate::instructions::sell::*;
use crate::instructions::transfer::*;
use crate::instructions::update_global::*;
use crate::instructions::set_sell_window::*;
use crate::instructions::migrate::*;

#[program]
pub mod sendor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::initialize(ctx)
    }

    pub fn create_launch(ctx: Context<CreateLaunch>, base_price: u64, slope: u64) -> Result<()> {
        crate::instructions::create_launch::create_launch(ctx, base_price, slope)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64, max_cost: u64) -> Result<()> {
        crate::instructions::buy::buy(ctx, amount, max_cost)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, min_payout: u64) -> Result<()> {
        crate::instructions::sell::sell(ctx, amount, min_payout)
    }

    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        crate::instructions::transfer::transfer(ctx, amount)
    }

    pub fn update_global(ctx: Context<UpdateGlobal>) -> Result<()> {
        crate::instructions::update_global::update_global(ctx)
    }

    /// Admin/cron: picks two 15-min windows 12-18 h apart each day.
    pub fn randomize_sell_window(ctx: Context<RandomizeSellWindow>) -> Result<()> {
        crate::instructions::set_sell_window::randomize_sell_window(ctx)
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        crate::instructions::migrate::migrate(ctx)
    }
}

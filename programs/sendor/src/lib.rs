use anchor_lang::prelude::*;

declare_id!("EEFRp8gpeh4seq6CDm3NFXPba6AGXZp5kVUuEuSgNaJs");

pub mod constants;
pub mod errors;
pub mod state {
    pub mod global_state;
    pub mod launch_metadata;
    pub mod bonding_curve_state;
    pub mod user_record;
}
pub mod instructions {
    pub mod initialize;
    pub mod create_launch;
    pub mod buy;
    pub mod sell;
    pub mod transfer;
    pub mod update_global;
    pub mod set_sell_window;
    pub mod migrate;
}

use crate::instructions::initialize::Initialize;
use crate::instructions::create_launch::CreateLaunch;
use crate::instructions::buy::Buy;
use crate::instructions::sell::Sell;
use crate::instructions::transfer::TransferTokens;
use crate::instructions::update_global::UpdateGlobal;
use crate::instructions::set_sell_window::SetSellWindow;
use crate::instructions::migrate::Migrate;

#[program]
pub mod sendor_dump {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    pub fn create_launch(
        ctx: Context<CreateLaunch>,
        base_price: u64,
        slope: u64
    ) -> Result<()> {
        instructions::create_launch::create_launch(ctx, base_price, slope)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64) -> Result<()> {
        instructions::buy::buy(ctx, amount)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64) -> Result<()> {
        instructions::sell::sell(ctx, amount)
    }

    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        instructions::transfer::transfer(ctx, amount)
    }

    pub fn update_global(ctx: Context<UpdateGlobal>) -> Result<()> {
        instructions::update_global::update_global(ctx)
    }

    pub fn set_sell_window(
        ctx: Context<SetSellWindow>,
        window1_start: i64,
        window2_start: i64
    ) -> Result<()> {
        instructions::set_sell_window::set_sell_window(ctx, window1_start, window2_start)
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        instructions::migrate::migrate(ctx)
    }
}

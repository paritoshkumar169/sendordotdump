use anchor_lang::prelude::*;

declare_id!("6mqsEaGREVXfAroU9WErmEPqYmKoFpoMHuFHzvBBGgna");

pub mod constants;
pub mod errors;
pub mod state;
pub mod instructions;

/* re-export every item (struct + constant) from each instruction module */
pub use instructions::initialize::*;
pub use instructions::create_launch::*;
pub use instructions::buy::*;
pub use instructions::sell::*;
pub use instructions::transfer::*;
pub use instructions::update_global::*;
pub use instructions::set_sell_window::*;
pub use instructions::migrate::*;

#[program]
pub mod sendor {
    use super::*;
    use crate::instructions::{
        buy, create_launch, initialize, migrate, sell, set_sell_window, transfer, update_global,
    };

    pub fn initialize(ctx: Context<Initialize>, platform_fee_recipient: Pubkey, launch_fee_lamports: u64) -> Result<()> {
        initialize::initialize(ctx, platform_fee_recipient, launch_fee_lamports)
    }

    pub fn create_launch(ctx: Context<CreateLaunch>, base_price: u64, slope: u64, token_name: String, token_symbol: String, token_uri: String) -> Result<()> {
        create_launch::create_launch(ctx, base_price, slope, token_name, token_symbol, token_uri)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64, max_cost: u64) -> Result<()> {
        buy::buy(ctx, amount, max_cost)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, min_payout: u64) -> Result<()> {
        sell::sell(ctx, amount, min_payout)
    }

    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        transfer::transfer(ctx, amount)
    }

    pub fn update_global(ctx: Context<UpdateGlobal>) -> Result<()> {
        update_global::update_global(ctx)
    }

    pub fn randomize_sell_window(ctx: Context<RandomizeSellWindow>) -> Result<()> {
        set_sell_window::randomize_sell_window(ctx)
    }

    pub fn migrate(ctx: Context<Migrate>) -> Result<()> {
        migrate::migrate(ctx)
    }
}

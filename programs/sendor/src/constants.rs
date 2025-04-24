use anchor_lang::prelude::*;


pub const TOKEN_DECIMALS: u8 = 9;                        // 9 decimal places for SPL token
pub const INITIAL_SUPPLY_TOKENS: u64 = 1_000_000_000;    // 1 billion tokens (whole tokens)
pub const INITIAL_SUPPLY_BASE_UNITS: u64 = INITIAL_SUPPLY_TOKENS * 1_000_000_000;  // 1e9 * 1e9 = 1e18 base units

pub const SELL_LIMIT_PERCENT: u64 = 10;      // 10% sell limit per day
pub const TRANSFER_LIMIT_PERCENT: u64 = 20;  // 20% transfer limit per day

pub const WINDOW_DURATION: i64 = 15 * 60;    // 15 minutes window duration (in seconds)
pub const HALF_DAY: i64 = 12 * 60 * 60;      // 12 hours in seconds (half-day interval)

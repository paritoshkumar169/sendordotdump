use anchor_lang::prelude::*;
use crate::errors::LaunchError;

#[account]
pub struct BondingCurveState {
    pub launch_metadata: Pubkey,
    pub base_price: u64,
    pub slope: u64,
    pub current_supply: u64,
    pub decimals: u8,
}

impl BondingCurveState {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 1;

    pub fn compute_payout(&self, amount: u64) -> Result<u64> {
        require!(self.decimals <= 18, LaunchError::InvalidDecimals);
        let m = 10u128.pow(self.decimals as u32);
        let base = self.base_price as u128;
        let slope = self.slope as u128;
        let s_end = self.current_supply as u128;
        let _s_start = s_end.checked_sub(amount as u128).ok_or(LaunchError::MathOverflow)?;
        let payout = base * amount as u128
            + slope * (s_end * amount as u128 - amount as u128 * (amount as u128 - 1) / 2);
        Ok((payout / m) as u64)
    }
}

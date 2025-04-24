use anchor_lang::prelude::*;

#[account]
pub struct BondingCurveState {

    pub launch_metadata: Pubkey,  
    pub base_price: u64,        
    pub slope: u64,              
    pub current_supply: u64,     
    pub decimals: u8,            
}

impl BondingCurveState {
    pub const LEN: usize = 8     // discriminator
        + 32                     // launch_metadata
        + 8                      // base_price
        + 8                      // slope
        + 8                      // current_supply
        + 1;                     // decimals
}

use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchError {
    #[msg("Trading is not allowed at this time.")]
    NotInTradingWindow,
    #[msg("This wallet has already performed an action in the current cycle.")]
    ActionAlreadyPerformed,
    #[msg("Sell amount exceeds the daily 10% limit of holdings.")]
    ExceedsSellLimit,
    #[msg("Transfer amount exceeds the daily 20% limit of holdings.")]
    ExceedsTransferLimit,
    #[msg("Insufficient token supply available for purchase.")]
    InsufficientSupply,
    #[msg("Insufficient funds to complete the purchase.")]
    InsufficientFunds,
    #[msg("Insufficient liquidity in pool for the sell amount.")]
    InsufficientLiquidity,
    #[msg("Unauthorized access or incorrect signer.")]
    Unauthorized,
    #[msg("Invalid trading window parameters.")]
    InvalidWindowTimes,
}

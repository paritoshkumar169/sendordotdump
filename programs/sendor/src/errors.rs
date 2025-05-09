use anchor_lang::prelude::*;

#[error_code]
pub enum LaunchError {
    #[msg("Trading window closed")]
    NotInTradingWindow,
    #[msg("Daily action already performed")]
    ActionAlreadyPerformed,
    #[msg("Sell exceeds 10 % limit")]
    ExceedsSellLimit,
    #[msg("Transfer exceeds 20 % limit")]
    ExceedsTransferLimit,
    #[msg("Decimals must be 18 or fewer")]
    InvalidDecimals,
    #[msg("Slippage limit hit")]
    SlippageExceeded,
    #[msg("Payout below min_payout")]
    PayoutTooLow,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Insufficient supply")]
    InsufficientSupply,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Unauthorized signer")]
    Unauthorized,
    #[msg("Invalid window parameters")]
    InvalidWindowTimes,
    #[msg("Migration too early")]
    PrematureMigration,
    #[msg("Invalid launch parameters")]
    InvalidParams,
    #[msg("Invalid fee recipient")]
    InvalidFeeRecipient,
}

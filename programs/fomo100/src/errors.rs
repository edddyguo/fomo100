use anchor_lang::{error_code, prelude::msg};

#[error_code]
pub enum StakeError {
    #[msg("stake amount must be more than 1000000 and has to be a multiple of 1000000")]
    StakeAmountInvalid,
    #[msg("stake amount is less than minimal")]
    LessThanMinimalStakeAmount,
    #[msg("insufficient balance")]
    InsufficientBalance,
    #[msg("stake mint not match")]
    NotMatchMint,
    #[msg("not allow unstake before end")]
    NotAllowUnstakeBeforeEnd,
    #[msg("Have Already Finished")]
    HaveAlreadyFinished,
    #[msg("Have Already Unstaked")]
    HaveAlreadyUnstaked,
    #[msg("Unknown")]
    Unknown,
}

pub fn unknown_error(e: &str) -> StakeError {
    msg!("unknown_error {}", e);
    StakeError::Unknown
}

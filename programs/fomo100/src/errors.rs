use anchor_lang::error_code;

#[error_code]
pub enum StakeError {
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

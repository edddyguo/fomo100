use anchor_lang::error_code;

#[error_code]
pub enum MinterError {
    #[msg("InSufficientFunds")]
    InSufficientFunds,
    #[msg("NotSupportCoin")]
    NotSupportCoin,
    #[msg("NftNotMatched")]
    NftNotMatched,
    #[msg("SellerNotMatched")]
    SellerNotMatched,
    #[msg("stake amount is less than minimal")]
    LessThanMinimalStakeAmount,
    #[msg("insufficient balance")]
    InsufficientBalance,
    #[msg("stake mint not match")]
    NotMatchMint,
    #[msg("insufficient claimed amount")]
    InsufficientClaimedAmount,
    #[msg("insufficient reward point")]
    InsufficientPoint,
    #[msg("invalid proof")]
    InvalidProof,
    #[msg("insufficient permission ")]
    InsufficientPermission,
    #[msg("no change")]
    NoChange,
    #[msg("invalid name")]
    InvalidName,
    #[msg("invalid symbol")]
    InvalidSymbol,
    #[msg("invalid uri")]
    InvalidUri,
    #[msg("not treasurer")]
    NotTreasurer,
    #[msg("SigVerificationFailed")]
    SigVerificationFailed,
    #[msg("UnknownError")]
    UnknownError,
}

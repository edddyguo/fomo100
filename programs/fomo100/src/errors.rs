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
    #[msg("Max Reward Records Exceeded")]
    MaxRewardRecordsExceeded,
    //权限不足
    #[msg("Permission Denied")]
    PermissionDenied,
    //已经解锁
    #[msg("Already Unlocked")]
    AlreadyUnlocked,
    //没有可领取奖励
    #[msg("Reward is zero")]
    RewardIsZero,
    //没有可领取奖励
    #[msg("Pool store is empty")]
    PoolStoreIsEmpty,
    //还未到解锁时间
    #[msg("UnlockTimeNotArrived")]
    UnlockTimeNotArrived,
    //还未解锁
    #[msg("NotUnlock")]
    NotUnlock,
    //用户质押为空
    #[msg("StakeIsEmpty")]
    StakeIsEmpty,
    //用户质押为空
    #[msg("PoolIsFinished")]
    PoolIsFinished,
    //用户质押为空
    #[msg("AlreadyUnstake")]
    AlreadyUnstake,
    //用户质押为空
    #[msg("BeyondStakeLimit")]
    BeyondStakeLimit,
    #[msg("Unknown")]
    Unknown,
}

pub fn unknown_error(e: &str) -> StakeError {
    msg!("unknown_error {}", e);
    StakeError::Unknown
}

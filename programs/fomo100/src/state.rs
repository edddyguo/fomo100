use anchor_lang::prelude::*;

//pub const MINIMAL_STAKE_AMOUNT: u64 = 1_000_000_000;
pub const MINIMAL_STAKE_AMOUNT: u64 = 1;
pub const ROUND_MAX: u32 = 2000;
//折衷的选择，允许用户累积100次天的快照，这是够用的，
//且当用户万一不够了，进行一次claim就行，这会删除之前的快照数据
pub const MAX_USER_STAKE_TIMES: u32 = 100;
pub const UNLOCK_DAYS: i64 = 30;

//todo:用户抵押的钱单独申请一个account，当前先放在pool_vault中
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_STATE_SEED: &str = "pool_state";
pub const USER_VAULT_SEED: &str = "user_vault";
pub const USER_STATE_SEED: &str = "user_state";

#[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UserStake {
    //质押时候对应的轮次
    pub round_index: u32,
    //对此对应的总质押量
    pub stake_amount: u64,
}
#[account]
#[derive(Debug)]
pub struct UserState {
    pub user: Pubkey,
    // 最多允许用户追加100次
    pub stakes: Vec<UserStake>,
    // 解锁那天的时间戳
    pub unlock_at: Option<i64>,
    // claimed_reward is null before claim
    pub claimed_reward: u64,
    pub is_unstaked: bool,
}

impl UserState {
    pub const LEN: usize = 32 + (4 + MAX_USER_STAKE_TIMES as usize * (4 + 8)) + (1 + 8) + 8 + 1;
}

#[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Round {
    pub index: u32,
    pub reward: u64,
    pub stake_amount: u64,
}
//todo: 重新计算内存分配量
#[account]
#[derive(Debug)]
pub struct PoolState {
    pub token_mint: Pubkey,
    pub round_period_secs: u32,
    pub created_at: i64,
    //当前轮次奖金池
    pub current_round_reward: u64,
    //允许跳空
    pub history_rounds: Vec<Round>,
    pub unlocking_users: u32,
    pub unlocking_stake_amount: u64,
    pub claimed_reward: u64,
}

impl PoolState {
    //初始化仅申请20个轮次的空间
    pub const LEN: usize =
        32 + 4 + 8 + 8 + (4 + (ROUND_MAX / 100) as usize * (4 + 8 + 8)) + 4 + 8 + 8;
}

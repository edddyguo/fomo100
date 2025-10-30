use anchor_lang::prelude::*;

//单位聪
pub const TOKEN_SCALE: u32 = 1_000_000;
//3 Year
pub const ROUND_MAX: usize = 1095;
//折衷的选择，允许用户累积100次天的快照，这是够用的，
//且当用户万一不够了，进行一次claim就行，这会删除之前的快照数据
pub const MAX_USER_STAKE_TIMES: usize = 100;
pub const UNLOCK_DAYS: i64 = 30;

//todo:用户抵押的钱单独申请一个account，当前先放在pool_vault中
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_STATE_SEED: &str = "pool_state";
pub const USER_VAULT_SEED: &str = "user_vault";
pub const USER_STATE_SEED: &str = "user_state";

#[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UserStake {
    //质押时候对应的自然轮次
    pub round_index: u16,
    //对此对应的总质押量,单位聪
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

// #[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
// pub struct Round {
//     pub index: u32,
//     pub reward: u64,
//     pub stake_amount: u64,
// }
#[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Round {
    //最多65535个自然轮
    pub index: u16,
    //轮次奖励，单位个
    //note: 虽然当前总供应量100亿理论上存在可能溢出，但实际上不会超过42亿，
    pub reward: u32,
    //用户质押总量，单位个
    pub stake_amount: u32,
}
//todo: 重新计算内存分配量
#[account]
#[derive(Debug)]
pub struct PoolState {
    pub token_mint: Pubkey,
    pub round_period_secs: u32,
    pub created_at: i64,
    //当前轮次奖金池,单位聪
    pub current_round_reward: u64,
    //允许跳空
    pub history_rounds: Vec<Round>,
    pub unlocking_users: u32,
    pub unlocking_stake_amount: u64,
    pub claimed_reward: u64,
}

impl PoolState {
    //初始化仅申请10个轮次的空间
    pub const LEN: usize =
        32 + 4 + 8 + 8 + (4 + (ROUND_MAX / 100) as usize * (4 + 8 + 8)) + 4 + 8 + 8;
    pub fn validate(&self) {
        //总的快照数超过超过限制则项目池子结束,
        //项目结束后仍允许stake之外的操作
        if self.history_rounds.len() > ROUND_MAX as usize {
            panic!("poll already finished")
        }
    }
}

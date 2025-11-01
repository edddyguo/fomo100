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
pub const POOL_STORE_SEED: &str = "pool_store";
pub const USER_VAULT_SEED: &str = "user_vault";
pub const USER_STATE_SEED: &str = "user_state";

#[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct UserStake {
    //质押时候对应的自然轮次
    pub round_index: u32,
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
use bytemuck::{Pod, Zeroable};
#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)] // 使用 C 语言的内存布局
pub struct Round {
    //轮次奖励，单位个
    //note: 虽然当前总供应量100亿理论上存在可能溢出，但实际上不会超过42亿，
    pub reward: u32,
    //用户质押总量，单位个
    pub stake_amount: u32,
    //最多65535个自然轮,
    pub index: u32,
}
#[account]
#[derive(Debug)]
pub struct PoolState {
    pub token_mint: Pubkey,
    pub round_period_secs: u32,
    pub unlocking_stake_amount: u64,
    pub claimed_reward: u64,
    pub created_at: i64,
    //当前轮次奖金池,单位聪
    pub current_round_reward: u64,
    pub unlocking_users: u32,
}

impl PoolState {
    //初始化仅申请10个轮次的空间
    //note:当前使用了zero_copy，不会用到该值
    //pub const LEN: usize = 32 + 4 + 8 + 8 + (4 + ROUND_MAX as usize * (2 + 4 + 4)) + 4 + 8 + 8;
    pub const LEN: usize = 32 + 4 + 8 + 8 + 8 + 8 + 4;
}

#[account(zero_copy)]
#[repr(C)]
pub struct PoolStore {
    pub len: u32, // 当前有效长度
    pub index: [u32; ROUND_MAX],
    pub reward: [u32; ROUND_MAX],
    pub stake_amount: [u32; ROUND_MAX],
}

impl PoolStore {
    pub fn new() -> Self {
        Self {
            reward: std::array::from_fn(|_| Default::default()),
            index: std::array::from_fn(|_| Default::default()),
            stake_amount: std::array::from_fn(|_| Default::default()),

            len: 0,
        }
    }

    // pub fn len(&self) -> usize {
    //     self.len as usize
    // }
    // pub fn is_empty(&self) -> bool {
    //     self.len == 0
    // }

    // /// 类似 Vec::push
    // pub fn push(&mut self, value: Round) -> Result<()> {
    //     if self.len() < ROUND_MAX {
    //         self.data[self.len()] = (value.index, value.reward, value.stake_amount);
    //         self.len += 1;
    //         Ok(())
    //     } else {
    //         Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into())
    //     }
    // }

    // /// 类似 Vec::last
    // pub fn last(&self) -> Option<&(u32, u32, u32)> {
    //     if self.len == 0 {
    //         None
    //     } else {
    //         Some(&self.data[self.len() - 1])
    //     }
    // }

    // /// 获取可变引用
    // pub fn last_mut(&mut self) -> Option<&mut (u32, u32, u32)> {
    //     if self.len == 0 {
    //         None
    //     } else {
    //         Some(&mut self.data[self.len() - 1])
    //     }
    // }

    // /// 根据 index 获取元素
    // pub fn get(&self, index: usize) -> Option<&(u32, u32, u32)> {
    //     if index < self.len() {
    //         Some(&self.data[index])
    //     } else {
    //         None
    //     }
    // }

    // /// 根据 index 获取可变元素
    // pub fn get_mut(&mut self, index: usize) -> Option<&mut (u32, u32, u32)> {
    //     if index < self.len() {
    //         Some(&mut self.data[index])
    //     } else {
    //         None
    //     }
    // }
}
// use std::ops::{Deref, RangeBounds};

// impl Deref for PoolStore {
//     type Target = [(u32, u32, u32)];

//     fn deref(&self) -> &Self::Target {
//         &self.data[..self.len()]
//     }
// }

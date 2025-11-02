use anchor_lang::prelude::*;

//单位聪
pub const TOKEN_SCALE: u32 = 1_000_000;
//3 Year
//note：为了内存对齐，此值必须是4的倍数
pub const ROUND_MAX: usize = 1096;
//折衷的选择，允许用户累积100次天的快照，这是够用的，
//超过150会在加载账号的时候报错内存溢出
pub const MAX_USER_STAKE_TIMES: usize = 300;
//最多设置100次奖励池子
pub const MAX_REWARD_RECORDS: usize = 100;
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
    pub const LEN: usize = 32 + (4 + MAX_USER_STAKE_TIMES * (2 + 8)) + (1 + 8) + 8 + 1;
}

// #[derive(Debug, Default, Clone, AnchorSerialize, AnchorDeserialize)]
// pub struct Round {
//     pub index: u32,
//     pub reward: u64,
//     pub stake_amount: u64,
// }
use bytemuck::{Pod, Zeroable};
#[derive(Debug, Default, Clone, Copy)]
pub struct Round {
    //轮次奖励，单位个
    //note: 虽然当前总供应量100亿理论上存在可能溢出，但实际上不会超过42亿，
    pub reward_index: u8,
    //用户质押总量，单位个
    pub stake_amount: u32,
    //最多65535个自然轮,
    pub round_index: u16,
}
#[account]
#[derive(Debug)]
pub struct PoolState {
    pub token_mint: Pubkey,
    pub admin: Pubkey,
    pub round_period_secs: u32,
    pub unlocking_stake_amount: u64,
    pub claimed_reward: u64,
    pub created_at: i64,
    //当前轮次奖金池,单位聪
    pub current_round_reward: u64,
    //历史轮次奖金记录，最多100次
    pub history_round_rewards: Vec<u64>,
    pub unlocking_users: u32,
}

impl PoolState {
    pub const LEN: usize = 32 + 32 + 4 + 8 + 8 + 8 + 8 + MAX_REWARD_RECORDS * 8 + 4;
}

#[account(zero_copy)]
#[derive(Debug)]
#[repr(C)]
pub struct PoolStore {
    //剔除pub属性
    pub reward_indexes: [u8; ROUND_MAX],
    pub round_indexes: [u16; ROUND_MAX],
    //最多更改256回的奖池资金
    pub stake_amounts: [u32; ROUND_MAX],
    pub len: u32, // 当前有效长度
}

impl PoolStore {
    pub const LEN: usize = 4 + 2 * ROUND_MAX + 1 * ROUND_MAX + 4 * ROUND_MAX;

    // pub fn new() -> Self {
    //     Self {
    //         // reward: std::array::from_fn(|_| Default::default()),
    //         index: std::array::from_fn(|_| Default::default()),
    //         // stake_amount: std::array::from_fn(|_| Default::default()),

    //         //len: 0,
    //     }
    // }

    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 类似 Vec::push
    pub fn push(&mut self, value: Round) -> Result<()> {
        if self.len() < ROUND_MAX {
            self.reward_indexes[self.len()] = value.reward_index;
            self.round_indexes[self.len()] = value.round_index;
            self.stake_amounts[self.len()] = value.stake_amount;
            self.len += 1;
            Ok(())
        } else {
            Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into())
        }
    }

    /// 类似 Vec::last
    pub fn last(&self) -> Option<Round> {
        if self.len == 0 {
            None
        } else {
            Some(Round {
                reward_index: self.reward_indexes[self.len as usize - 1],
                stake_amount: self.stake_amounts[self.len as usize - 1],
                round_index: self.round_indexes[self.len as usize - 1],
            })
        }
    }

    // /// 获取可变引用
    pub fn last_reward_index_mut(&mut self) -> Option<&mut u8> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.reward_indexes[self.len() - 1])
        }
    }

    pub fn last_stake_amount_mut(&mut self) -> Option<&mut u32> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.stake_amounts[self.len() - 1])
        }
    }

    pub fn last_round_index_mut(&mut self) -> Option<&mut u16> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.round_indexes[self.len() - 1])
        }
    }

    //获取有效的轮次，即非零值
    pub fn round_indexes(&self) -> &[u16] {
        match self.round_indexes.iter().position(|x| *x == 0) {
            Some(0) => &[],
            Some(zero_position) => &self.round_indexes[0..zero_position],
            None => &self.round_indexes,
        }
    }

    //获取有效的轮次，即非零值
    //根据round_index是否存在来判断插入或者更新
    //更新时，对应index是
    // pub fn insert_or_update(&self) -> &[u16] {
    //     let zero_position = self.round_indexes.iter().position(|x| *x == 0).unwrap();
    //     if zero_position == 0 {
    //         &[]
    //     } else {
    //         &self.round_indexes[0..zero_position]
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

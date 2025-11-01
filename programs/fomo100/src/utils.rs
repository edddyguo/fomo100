use crate::errors::StakeError;
use anchor_lang::prelude::*;
use spl_token::solana_program::ed25519_program::ID as ED25519_ID;
use spl_token::solana_program::instruction::Instruction;

use crate::state::{PoolState, Round, UserStake, MAX_USER_STAKE_TIMES, ROUND_MAX, TOKEN_SCALE};
use std::convert::TryInto;
use std::ops::{Deref, RangeBounds};

pub const DAY1: i64 = 60 * 60 * 24;

//获取当前轮次,
pub fn current_round_index(pool_init: i64) -> Result<i64> {
    let clock = Clock::get()?;
    let index = (clock.unix_timestamp - pool_init) / DAY1;
    Ok(index)
}

//将质押的记录，展开为对应轮次的记录,
//展开后用户轮次和pool轮次保持一致，
//hack: to optimize, 没必要全部展开，可以在使用的时候加上，对应index没有的话，就使用上一个轮次的值，这个逻辑
pub fn flatten_user_stake_snap(current_round_index: u32, user_stakes: &Vec<UserStake>) -> Vec<u64> {
    let mut stake_snaps = vec![];
    //如果用户没有质押，则历史值全为零
    let mut last_stake_amount = user_stakes.last().map(|x| x.stake_amount).unwrap_or(0);

    if user_stakes
        .iter()
        .any(|x| x.round_index > current_round_index)
    {
        //Err(StakeError::Unknown)?;
        panic!("it is unreachable,user's round index must less than pool");
    }

    for index in 0..current_round_index {
        let x: Vec<&UserStake> = user_stakes
            .iter()
            .filter(|x| x.round_index == index)
            .collect();

        match x.as_slice() {
            [stake] => {
                stake_snaps.push(stake.stake_amount);
                last_stake_amount = stake.stake_amount;
            }
            [] => stake_snaps.push(last_stake_amount),
            _ => {
                panic!("user have multi stake record in a round");
            }
        }
    }
    stake_snaps
}

//hack: to optimize, 没必要全部展开，可以在使用的时候加上，对应index没有的话，就使用上一个轮次的值，这个逻辑
//返回全历史记录的（奖励和总质押）
// pub fn flatten_pool_stake_snap(
//     current_round_index: u16,
//     pool_stakes: &Vec<Round>,
// ) -> Vec<(u32, u32)> {
//     let mut stake_snaps = vec![];
//     //如果用户没有质押，则历史值全为零
//     let (mut last_stake_amount, mut last_reward) = pool_stakes
//         .last()
//         .map(|x| (x.stake_amount, x.reward))
//         .unwrap_or((0, 0));

//     if pool_stakes.iter().any(|x| x.index > current_round_index) {
//         //Err(StakeError::Unknown)?;
//         panic!("it is unreachable,user's round index must less than pool");
//     }

//     for index in 0..current_round_index {
//         let x: Vec<&Round> = pool_stakes.iter().filter(|x| x.index == index).collect();

//         match x.as_slice() {
//             [stake] => {
//                 stake_snaps.push((stake.stake_amount, stake.reward));
//                 last_stake_amount = stake.stake_amount;
//                 last_reward = stake.reward;
//             }
//             [] => stake_snaps.push((last_stake_amount, last_reward)),
//             _ => {
//                 panic!("user have multi stake record in a round");
//             }
//         }
//     }
//     stake_snaps
// }

//根据轮次历史快照和用户stake的历史记录，计算总的奖励金额
pub fn calculate_total_reward(pool_rounds: &[Round], user_stakes: &Vec<UserStake>) -> Result<u64> {
    // 至少要有2个轮次（最后一个未结束）
    if pool_rounds.len() <= 1 || user_stakes.len() <= 1 {
        return Ok(0);
    }

    let pool_rounds = &pool_rounds[..pool_rounds.len() - 1];
    let user_stakes = &user_stakes[..user_stakes.len() - 1];

    let mut total_reward: u128 = 0;
    let mut i = 0usize; // pool_rounds 指针

    for user_stake in user_stakes {
        // 顺序扫描直到找到对应 round_index
        while i < pool_rounds.len() && pool_rounds[i].index < user_stake.round_index as u32 {
            i += 1;
        }

        // 找不到直接 panic（逻辑上不该发生）
        assert!(
            i < pool_rounds.len() && pool_rounds[i].index == user_stake.round_index as u32,
            "Pool round for index {} not found (user state corrupted)",
            user_stake.round_index
        );

        let round = &pool_rounds[i];
        if round.stake_amount == 0 {
            panic!(
                "Pool round {} stake_amount == 0, invalid state",
                round.index
            );
        }

        let reward = (user_stake.stake_amount as u128) * (round.reward as u128)
            / (round.stake_amount as u128);
        total_reward += reward;
    }

    Ok(total_reward as u64)
}

//向下取整
pub fn get_current_round_index(init_at: i64, now: i64, period: u32) -> u32 {
    ((now - init_at) / (period as i64)) as u32
}

use bytemuck::{Pod, Zeroable};

//FixedVec 基于数组实现
// #[derive(Debug, Clone, Copy, Zeroable, Pod)]
// #[repr(C)] // 使用 C 语言的内存布局
// pub struct FixedVec<T: Default + Copy, const N: usize> {
//     pub len: u32, // 当前有效长度
//     pub data: [T; N],
// }

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)] // 强制无填充
pub struct RoundSnaps {
    pub len: u32, // 当前有效长度
    pub data: [Round; ROUND_MAX],
}

impl RoundSnaps {
    pub fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| Round::default()),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 类似 Vec::push
    pub fn push(&mut self, value: Round) -> Result<()> {
        if self.len() < ROUND_MAX {
            self.data[self.len()] = value;
            self.len += 1;
            Ok(())
        } else {
            Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into())
        }
    }

    /// 类似 Vec::last
    pub fn last(&self) -> Option<&Round> {
        if self.len == 0 {
            None
        } else {
            Some(&self.data[self.len() - 1])
        }
    }

    /// 获取可变引用
    pub fn last_mut(&mut self) -> Option<&mut Round> {
        if self.len == 0 {
            None
        } else {
            Some(&mut self.data[self.len() - 1])
        }
    }

    /// 根据 index 获取元素
    pub fn get(&self, index: usize) -> Option<&Round> {
        if index < self.len() {
            Some(&self.data[index])
        } else {
            None
        }
    }

    /// 根据 index 获取可变元素
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Round> {
        if index < self.len() {
            Some(&mut self.data[index])
        } else {
            None
        }
    }
}

impl Deref for RoundSnaps {
    type Target = [Round];

    fn deref(&self) -> &Self::Target {
        &self.data[..self.len()]
    }
}

pub trait AmountRaw {
    fn raw(self) -> u64;
}

impl AmountRaw for u32 {
    fn raw(self) -> u64 {
        self as u64 * TOKEN_SCALE as u64
    }
}

pub trait AmountView {
    fn view(self) -> u32;
}

impl AmountView for u64 {
    fn view(self) -> u32 {
        (self / TOKEN_SCALE as u64) as u32
    }
}

fn tuples_to_round_slice(tuples: &[(u32, u32, u32)]) -> &[Round] {
    unsafe {
        // 强制转换 slice 的指针
        std::slice::from_raw_parts(tuples.as_ptr() as *const Round, tuples.len())
    }
}

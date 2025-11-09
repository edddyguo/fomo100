use crate::errors::StakeError;
use anchor_lang::prelude::*;
use spl_token::solana_program::ed25519_program::ID as ED25519_ID;
use spl_token::solana_program::instruction::Instruction;

use crate::state::{PoolState, PoolStore, Round, UserStake, MAX_USER_STAKE_TIMES, ROUND_MAX};
use std::collections::HashMap;
use std::convert::TryInto;
use std::ops::{Deref, RangeBounds};

pub const DAY1: i64 = 60 * 60 * 24;

//获取当前轮次,
pub fn current_round_index(pool_init: i64) -> Result<i64> {
    let clock = Clock::get()?;
    let index = (clock.unix_timestamp - pool_init) / DAY1;
    Ok(index)
}

//废弃，太消耗内存
//将质押的记录，展开为对应轮次的记录,
//展开后用户轮次和pool轮次保持一致，
// pub fn flatten_user_stake_snap(current_round_index: u16, user_stakes: &Vec<UserStake>) -> Vec<u64> {
//     let mut stake_snaps = vec![];
//     //如果用户没有质押，则历史值全为零
//     let mut last_stake_amount = user_stakes.last().map(|x| x.stake_amount).unwrap_or(0);

//     if user_stakes
//         .iter()
//         .any(|x| x.round_index > current_round_index)
//     {
//         //Err(StakeError::Unknown)?;
//         panic!("it is unreachable,user's round index must less than pool");
//     }

//     for index in 0..current_round_index {
//         let x: Vec<&UserStake> = user_stakes
//             .iter()
//             .filter(|x| x.round_index == index)
//             .collect();

//         match x.as_slice() {
//             [stake] => {
//                 stake_snaps.push(stake.stake_amount);
//                 last_stake_amount = stake.stake_amount;
//             }
//             [] => stake_snaps.push(last_stake_amount),
//             _ => {
//                 panic!("user have multi stake record in a round");
//             }
//         }
//     }
//     stake_snaps
// }

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
//todo; 如果要简化逻辑，可以做出，stake当天不允许用户claim
pub fn calculate_total_reward(
    current_round_index: u16,
    pool_state: &PoolState,
    pool_store: &PoolStore,
    user_stakes: &Vec<UserStake>,
) -> Result<u64> {
    // 至少要有2个轮次（最后一个未结束）
    if pool_store.is_empty() || user_stakes.is_empty() {
        return Ok(0);
    }
    //获取用户有效轮次,处在当前自然轮次的用户，不计算奖励
    let valid_user_stakes = if user_stakes.last().unwrap().round_index == current_round_index {
        //用户首次质押的轮次，奖励也为0
        if user_stakes.len() == 1 {
            return Ok(0);
        } else {
            &user_stakes[..user_stakes.len() - 1]
        }
    } else {
        user_stakes
    };
    //msg!("user_stakes {:?}", valid_user_stakes);

    let user_stake_map: HashMap<u16, u64> = valid_user_stakes
        .iter()
        .map(|x| (x.round_index, x.stake_amount))
        .collect();
    //计算总收益
    let mut total_reward: u64 = 0;
    let mut last_user_stake_amount = 0u64;
    let mut last_round_reward = 0u64;

    //最多1096个循环
    for (actual_index, natural_index) in pool_store.round_indexes().iter().enumerate() {
        //首个快照不需要计算跳空值
        if actual_index != 0 {
            let round_skip_num = natural_index - pool_store.round_indexes[actual_index - 1];
            //连续的轮次跳空奖励为0
            total_reward += (round_skip_num - 1) as u64 * last_round_reward;
            // println!(
            //     "add total_reward {},round_skip_num = {},last_round_reward={}",
            //     (round_skip_num - 1) as u64 * last_round_reward,
            //     round_skip_num,
            //     last_round_reward,
            // )
        }

        //找到则使用用户对应轮次的质押值，并更新last_user_stake_amount,否则则使用上一轮次的值,
        let current_stake_amount =
            if let Some(user_stake_amount) = user_stake_map.get(natural_index) {
                last_user_stake_amount = *user_stake_amount;
                *user_stake_amount
            } else {
                //如果找不到抵押记录、且前值为零说明是之前的轮次用户没有曾参与
                if last_user_stake_amount == 0 {
                    continue;
                } else {
                    last_user_stake_amount
                }
            };

        //获取奖池金额下标
        let reward_index = pool_store.reward_indexes[actual_index as usize];
        //获取奖池金额
        let pool_round_reward = pool_state.history_round_rewards[reward_index as usize];
        let reward = calculate_user_reward(
            pool_round_reward,
            pool_store.stake_amounts[actual_index],
            current_stake_amount,
            pool_state.token_scale,
        );
        //累加总奖励，更新last_round_reward值
        total_reward += reward;
        // println!(
        //     "add total_reward {},pool_round_reward = {},pool_stake_amounts={},user_stake_amount={}",
        //     reward, pool_round_reward, pool_store.stake_amounts[actual_index], current_stake_amount
        // );
        last_round_reward = reward;
    }
    //最后一次快照到当前为止的剩余有效轮次，比如最后一次快照在自然轮次12，当前自然轮次为15，则剩下的有效奖励轮次为 2，即（第13和第14）
    let last_round_index = *pool_store.round_indexes().last().unwrap();
    //如果自然轮次超过最后一个快照的轮次，收益也截止到最后一个快照轮次
    let current_round_index = if current_round_index > pool_store.round_indexes[ROUND_MAX - 1] {
        pool_store.round_indexes[ROUND_MAX - 1] + 1
    } else {
        current_round_index
    };
    if current_round_index > last_round_index {
        let remainder_natural_num = current_round_index - last_round_index - 1;

        total_reward += last_round_reward * remainder_natural_num as u64;
        // println!(
        //     "add total_reward {},last_round_reward = {},remainder_natural_num={},current_round_index={}.last_round_index={}",
        //     last_round_reward * remainder_natural_num as u64,
        //     last_round_reward,
        //     remainder_natural_num,
        //     current_round_index,
        //     last_round_index,
        // );
    }

    Ok(total_reward)
}

//向下取整
pub fn get_current_round_index(init_at: i64, now: i64, period: u32) -> u16 {
    ((now - init_at) / (period as i64)) as u16
}

//获取用奖励值,（用户质押 / 奖池质押） * 奖池金额
pub fn calculate_user_reward(
    round_reward: u64,
    round_stake_amount: u32,
    user_stake_amount: u64,
    token_scale: u64,
) -> u64 {
    ((user_stake_amount as u128) * (round_reward as u128)
        / round_stake_amount.raw(token_scale) as u128) as u64
}

use bytemuck::{Pod, Zeroable};

//FixedVec 基于数组实现
// #[derive(Debug, Clone, Copy, Zeroable, Pod)]
// #[repr(C)] // 使用 C 语言的内存布局
// pub struct FixedVec<T: Default + Copy, const N: usize> {
//     pub len: u32, // 当前有效长度
//     pub data: [T; N],
// }

// #[derive(Debug, Clone, Copy, Zeroable, Pod)]
// #[repr(C)] // 强制无填充
// pub struct RoundSnaps {
//     pub len: u32, // 当前有效长度
//     pub data: [Round; ROUND_MAX],
// }

// impl RoundSnaps {
//     pub fn new() -> Self {
//         Self {
//             data: std::array::from_fn(|_| Round::default()),
//             len: 0,
//         }
//     }

//     pub fn len(&self) -> usize {
//         self.len as usize
//     }
//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }

//     /// 类似 Vec::push
//     pub fn push(&mut self, value: Round) -> Result<()> {
//         if self.len() < ROUND_MAX {
//             self.data[self.len()] = value;
//             self.len += 1;
//             Ok(())
//         } else {
//             Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into())
//         }
//     }

//     /// 类似 Vec::last
//     pub fn last(&self) -> Option<&Round> {
//         if self.len == 0 {
//             None
//         } else {
//             Some(&self.data[self.len() - 1])
//         }
//     }

//     /// 获取可变引用
//     pub fn last_mut(&mut self) -> Option<&mut Round> {
//         if self.len == 0 {
//             None
//         } else {
//             Some(&mut self.data[self.len() - 1])
//         }
//     }

//     /// 根据 index 获取元素
//     pub fn get(&self, index: usize) -> Option<&Round> {
//         if index < self.len() {
//             Some(&self.data[index])
//         } else {
//             None
//         }
//     }

//     /// 根据 index 获取可变元素
//     pub fn get_mut(&mut self, index: usize) -> Option<&mut Round> {
//         if index < self.len() {
//             Some(&mut self.data[index])
//         } else {
//             None
//         }
//     }
// }

// impl Deref for RoundSnaps {
//     type Target = [Round];

//     fn deref(&self) -> &Self::Target {
//         &self.data[..self.len()]
//     }
// }

pub trait AmountRaw {
    fn raw(&self, scale: u64) -> u64;
}

impl AmountRaw for u32 {
    fn raw(&self, scale: u64) -> u64 {
        (*self as u64) * scale
    }
}

pub trait AmountView {
    fn view(&self, scale: u64) -> u32;
}

impl AmountView for u64 {
    fn view(&self, scale: u64) -> u32 {
        (self / scale) as u32
    }
}

fn tuples_to_round_slice(tuples: &[(u32, u32, u32)]) -> &[Round] {
    unsafe {
        // 强制转换 slice 的指针
        std::slice::from_raw_parts(tuples.as_ptr() as *const Round, tuples.len())
    }
}

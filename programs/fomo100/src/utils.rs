use crate::errors::StakeError;
use anchor_lang::prelude::*;
use spl_token::solana_program::ed25519_program::ID as ED25519_ID;
use spl_token::solana_program::instruction::Instruction;

use crate::state::{Round, UserStake};
use std::convert::TryInto;
use std::ops::RangeBounds;

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
pub fn flatten_pool_stake_snap(
    current_round_index: u32,
    pool_stakes: &Vec<Round>,
) -> Vec<(u64, u64)> {
    let mut stake_snaps = vec![];
    //如果用户没有质押，则历史值全为零
    let (mut last_stake_amount, mut last_reward) = pool_stakes
        .last()
        .map(|x| (x.stake_amount, x.reward))
        .unwrap_or((0, 0));

    if pool_stakes.iter().any(|x| x.index > current_round_index) {
        //Err(StakeError::Unknown)?;
        panic!("it is unreachable,user's round index must less than pool");
    }

    for index in 0..current_round_index {
        let x: Vec<&Round> = pool_stakes.iter().filter(|x| x.index == index).collect();

        match x.as_slice() {
            [stake] => {
                stake_snaps.push((stake.stake_amount, stake.reward));
                last_stake_amount = stake.stake_amount;
                last_reward = stake.reward;
            }
            [] => stake_snaps.push((last_stake_amount, last_reward)),
            _ => {
                panic!("user have multi stake record in a round");
            }
        }
    }
    stake_snaps
}

//根据轮次历史快照和用户stake的历史记录，计算总的奖励金额
pub fn calculate_total_reward(
    current_round_index: u32,
    pool_rounds: &Vec<Round>,
    user_stakes: &Vec<UserStake>,
) -> Result<u64> {
    let mut pool_stake_snip = flatten_pool_stake_snap(current_round_index, pool_rounds);
    let mut user_stake_snip = flatten_user_stake_snap(current_round_index, &user_stakes);
    assert_eq!(pool_rounds.len(), user_stake_snip.len());
    //当前轮次不产生奖励，剔除
    pool_stake_snip.pop();
    user_stake_snip.pop();
    let mut total_reward = 0;
    for (round, user_stake_amount) in pool_rounds.into_iter().zip(user_stake_snip.into_iter()) {
        //中间值也许会超过u64，但最终结果肯定在u64范围内
        let round_reward =
            user_stake_amount as u128 * round.reward as u128 / round.stake_amount as u128;
        total_reward += round_reward as u64;
    }
    Ok(total_reward)
}

//向下取整
pub fn get_current_round_index(init_at: i64, now: i64, period: u32) -> u32 {
    ((now - init_at) / (period as i64)) as u32
}

use crate::{errors::StakeError, state::*, utils::get_current_round_index};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;

//创建指定开始时间的池子，设置轮次周期
pub fn handler(ctx: Context<SetRoundReward>, round_reward: u64) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
    let pool_store = &mut ctx.accounts.pool_store.load_mut()?;

    //检查是否超过100次
    require_gt!(
        MAX_REWARD_RECORDS,
        pool_state.history_round_rewards.len(),
        StakeError::MaxRewardRecordsExceeded
    );
    //检查是否是管理员
    require_eq!(
        pool_state.admin,
        ctx.accounts.admin.key(),
        StakeError::PermissionDenied
    );
    let clock = Clock::get()?;
    let current_round_index = get_current_round_index(
        pool_state.created_at,
        clock.unix_timestamp,
        pool_state.round_period_secs,
    );
    let rewards_len = pool_state.history_round_rewards.len() as u8;
    pool_store.create_or_update_snap(current_round_index, Some(rewards_len), None);
    pool_state.history_round_rewards.push(round_reward);
    pool_state.current_round_reward = round_reward;

    Ok(())
}

#[derive(Accounts)]
pub struct SetRoundReward<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,
    /// 池子历史快照
    #[account(mut)]
    pub pool_store: AccountLoader<'info, PoolStore>,
    pub system_program: Program<'info, System>,
}

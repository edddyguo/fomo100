use crate::{errors::StakeError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;

//创建指定开始时间的池子，设置轮次周期
pub fn handler(ctx: Context<SetRoundReward>, round_reward: u64) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
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
    pool_state.history_round_rewards.push(round_reward);
    Ok(())
}

#[derive(Accounts)]
pub struct SetRoundReward<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,
    pub system_program: Program<'info, System>,
}

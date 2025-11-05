use std::{u16, u32};

use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;
use crate::errors::StakeError;

//创建指定开始时间的池子，设置轮次周期
pub fn handler(
    ctx: Context<CreatePool>,
    token_decimal: u8,
    min_stake_amount: u64,
    created_at: i64,
    round_period_secs: u32,
    round_reward: u64,
    unlock_period_secs: u64,
) -> Result<()> {
    let pool_store = &mut ctx.accounts.pool_store.load_init()?;
    pool_store.len = 0;
    //默认值为u16::Max规避0轮次问题
    pool_store.round_indexes = std::array::from_fn(|_| u16::MAX);
    pool_store.reward_indexes = std::array::from_fn(|_| u8::MAX);
    pool_store.stake_amounts = std::array::from_fn(|_| u32::MAX);

    let token_scale = 10u64.pow(token_decimal.into());
    //最小值应大于token精度
    if min_stake_amount < token_scale {
        Err(StakeError::StakeAmountInvalid)?;
    }

    let pool_state = &mut ctx.accounts.pool_state;

    pool_state.admin = ctx.accounts.admin.key();

    pool_state.token_mint = ctx.accounts.token_mint.key();

    pool_state.token_scale = token_scale;

    pool_state.min_stake_amount = min_stake_amount;

    pool_state.round_period_secs = round_period_secs;

    pool_state.unlock_period_secs = unlock_period_secs;

    pool_state.unlocking_stake_amount = 0;

    pool_state.claimed_reward = 0;


    pool_state.created_at = created_at;

    pool_state.current_round_reward = 0;

    pool_state.unlocking_users = 0;

    pool_state.history_round_rewards = vec![round_reward];

    msg!(
        "Initialize pool {}",
        ctx.accounts.pool_state.to_account_info().key()
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(token_decimal:u8,min_stake_amount:u64,created_at:i64,round_period_secs: u32,round_reward:u64,unlock_period_secs:u64)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    //init pool state by ended_at
    #[account(
        init,
        payer=admin,
        seeds=[token_mint.key().as_ref(),created_at.to_be_bytes().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STATE_SEED.as_bytes()],
        bump,
        space=8 + PoolState::LEN
    )]
    pub pool_state: Account<'info, PoolState>,
    #[account(
        //zero
        init,
        payer=admin,
        seeds=[token_mint.key().as_ref(),created_at.to_be_bytes().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STORE_SEED.as_bytes()], 
        bump,
        space=8 + PoolStore::LEN
    )]
    pub pool_store: AccountLoader<'info, PoolStore>,
    #[account(
        init,
        associated_token::mint = token_mint,
        associated_token::authority = pool_state,
        payer = admin,
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

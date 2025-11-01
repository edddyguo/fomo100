use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;

//创建指定开始时间的池子，设置轮次周期
pub fn handler(
    ctx: Context<CreatePool>,
    created_at: i64,
    round_period_secs: u32,
    round_reward: u64,
) -> Result<()> {
    let pool_store = &mut ctx.accounts.pool_store.load_init()?;
    pool_store.len = 0;
    pool_store.round_index = std::array::from_fn(|_| Default::default());
    pool_store.reward_index = std::array::from_fn(|_| Default::default());
    pool_store.stake_amount = std::array::from_fn(|_| Default::default());

    let pool_state = &mut ctx.accounts.pool_state;

    pool_state.token_mint = ctx.accounts.token_mint.key();

    pool_state.round_period_secs = round_period_secs;

    pool_state.created_at = created_at;

    pool_state.current_round_reward = 0;

    pool_state.unlocking_users = 0;

    pool_state.unlocking_stake_amount = 0;

    pool_state.claimed_reward = 0;

    pool_state.history_round_rewards = vec![round_reward];
    //设置管理员
    pool_state.admin = ctx.accounts.admin.key();

    msg!(
        "Initialize pool {}",
        ctx.accounts.pool_state.to_account_info().key()
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(created_at:i64,round_period_secs: u32)]
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

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use instructions::*;
use mpl_token_metadata::types::Collection;
use std::str::FromStr;

declare_id!("33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ");

#[program]
pub mod fomo100 {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        instructions::claim::handler(ctx)
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        created_at: i64,
        round_period_secs: u32,
        round_reward: u64,
    ) -> Result<()> {
        instructions::create_pool::handler(ctx, created_at, round_period_secs, round_reward)
    }

    pub fn expand_pool_state(ctx: Context<ExpandPoolState>) -> Result<()> {
        instructions::expand_pool_state::handler(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake::handler(ctx, amount)
    }

    pub fn unlock(ctx: Context<Unlock>, created_at: i64, round_period_secs: u32) -> Result<()> {
        instructions::unlock::handler(ctx, created_at, round_period_secs)
    }

    pub fn unstake(ctx: Context<Unstake>, created_at: i64, round_period_secs: u32) -> Result<()> {
        instructions::unstake::handler(ctx, created_at, round_period_secs)
    }

    pub fn update_pool(ctx: Context<Unstake>, reward: i64, owner: Vec<Pubkey>) -> Result<()> {
        todo!()
    }

    pub fn delegate_stake(ctx: Context<Unstake>, stake_start_at: i64, amount: u64) -> Result<()> {
        //todo
        //1、质押用户未进行过质押的情况下（自主质押或者已经进行过委托质押），才允许委托
        todo!()
    }
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub puppet: Account<'info, Data>,
}

#[account]
pub struct Data {
    pub data: u64,
}

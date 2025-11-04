use std::ops::Div;

use crate::utils::{get_current_round_index, AmountView};
use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

pub fn handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    msg!("file {}, line: {}", file!(), line!());
    if amount < TOKEN_SCALE as u64 || amount % (TOKEN_SCALE as u64) != 0 {
        Err(StakeError::StakeAmountInvalid)?;
    }

    let pool_state = &mut ctx.accounts.pool_state;
    let pool_store = &mut ctx.accounts.pool_store.load_mut()?;
    let user_state = &mut ctx.accounts.user_state;

    msg!("file {}, line: {}", file!(), line!());
    msg!("pool_store.round_snaps.len(): {}", pool_store.len());
    msg!("user_state.stakes.len(): {}", user_state.stakes.len());

    if user_state.is_unstaked {
        Err(StakeError::AlreadyUnstake)?;
    }

    if user_state.unlock_at.is_some(){
        Err( StakeError::AlreadyUnlocked)?;
    }

    if user_state.stakes.len() > MAX_USER_STAKE_TIMES{
        Err( StakeError::BeyondStakeLimit)?;
    }

    let clock = Clock::get()?;
    let current_round_index = get_current_round_index(
        pool_state.created_at,
        clock.unix_timestamp,
        pool_state.round_period_secs,
    );
 
    msg!("file {}, line: {}", file!(), line!());

    //整个池子的首次stake
    //let current_reward_index = pool_state.history_round_rewards.len() as u8 - 1;

    if pool_store.is_empty() {
        //reward 直接继承
        pool_store.push(Round {
            round_index: current_round_index,
            reward_index: pool_state.history_round_rewards.len() as u8  - 1,
            stake_amount: amount.view(),
        })?;
    //如果history_rounds的最后一个值等于current_round_index则说明，d当前轮次已经创建，直接更新即可
    } else {
        let last_round = pool_store.last().expect("must have a item");
        if last_round.round_index == current_round_index {
            *pool_store.last_stake_amount_mut().unwrap() += amount.view();
        //如果history_rounds的最后一个值不等于current_round_index则说明，当前为这个轮次的第一个stake
        } else if last_round.round_index < current_round_index {
            //本地快照初始化继承上一轮的stake_amount值基础上增加当前用户质押数量
            let stake_amount = last_round.stake_amount + amount.view();
            pool_store.push(Round {
                round_index: current_round_index,
                reward_index: last_round.reward_index,
                stake_amount,
            })?;
        } else {
            unreachable!("pool_store.last().unwrap().round_index {},current_round_index {}",pool_store.last().unwrap().round_index,current_round_index);
        }
    }

    msg!("file {}, line: {}", file!(), line!());

    //update user state
    require_gt!(
        MAX_USER_STAKE_TIMES,
        user_state.stakes.len(),
        StakeError::Unknown
    );
    user_state.user = ctx.accounts.user.key();
    msg!("file {}, line: {}", file!(), line!());

    //如果首次质押，则历史质押值为0
    let default_stake = &UserStake::default();
    let newest_stake_amount = user_state
        .stakes
        .last()
        .unwrap_or(&default_stake)
        .stake_amount
        + amount;
    let user_stake = UserStake {
        round_index: current_round_index,
        stake_amount: newest_stake_amount,
    };
    msg!("file {}, line: {}", file!(), line!());

    match user_state.stakes.last_mut() {
        // //当前轮次，首次,
        Some(stake) if stake.round_index < current_round_index => {
            user_state.stakes.push(user_stake);
        }
        //当前轮次，多次质押
        Some(stake) if stake.round_index == current_round_index => {
            stake.stake_amount = newest_stake_amount;
        }
        //首次质押
        None => {
            user_state.stakes.push(user_stake);
        }
        _ => {
            unreachable!("{} {}", line!(), file!());
        }
    }
    msg!("file {}, line: {}", file!(), line!());

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_ata.to_account_info(),
        to: ctx.accounts.user_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    msg!("file {}, line: {}", file!(), line!());
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;
    msg!("file {}, line: {}", file!(), line!());

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// 奖池状态
    #[account(mut)]
    pub pool_state: Box<Account<'info, PoolState>>,
    ///奖池资金库
    #[account(
        mut, 
        associated_token::mint = token_mint,
        associated_token::authority = pool_state
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,
    /// 用户状态
    #[account(
        init_if_needed,
        payer=user,
        seeds=[user.key().as_ref(),pool_state.key().as_ref(), USER_STATE_SEED.as_bytes()], 
        bump,
        space = 8 + UserState::LEN
    )]
    pub user_state: Box<Account<'info, UserState>>,
    /// 用户在合约的财库
    #[account(
        init_if_needed,
        payer=user, 
        associated_token::mint = token_mint,
        associated_token::authority = user_state
    )]
    pub user_vault: InterfaceAccount<'info, TokenAccount>,
    /// 池子历史快照
    #[account(mut)]
    pub pool_store: AccountLoader<'info, PoolStore>,
    /// 用户ata
    #[account(mut,associated_token::mint = token_mint,associated_token::authority = user)]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = token_mint.key() == pool_state.token_mint @ StakeError::NotMatchMint)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

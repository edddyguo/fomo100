use std::ops::Div;

use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::utils::{ get_current_round_index, AmountView};

pub fn handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    // msg!("file {}, line: {}", file!(), line!());
    // if amount < TOKEN_SCALE as u64 || amount % (TOKEN_SCALE as u64)  != 0{
    //     Err(StakeError::StakeAmountInvalid)?;
    // }

    // let pool_state = &mut ctx.accounts.pool_state;
    // let pool_store = &mut ctx.accounts.pool_store.load_init()?;
    // let user_state = &mut ctx.accounts.user_state;
    // //todo; 用固定长度数据来处理，不然每次都要reserve
    // //user_state.stakes.reserve(MAX_USER_STAKE_TIMES as usize);

    // msg!("file {}, line: {}", file!(), line!());
    // msg!("pool_store.round_snaps.len(): {}", pool_store.len());
    // msg!("user_state.stakes.len(): {}", user_state.stakes.len());


    // let clock = Clock::get()?;
    // let current_round_index = get_current_round_index(pool_state.created_at,clock.unix_timestamp,pool_state.round_period_secs); 
    // //已解锁的禁止再质押
    // //todo: 更多错误码
    // if user_state.unlock_at.is_some(){
    //     Err( StakeError::Unknown)?;
    // }
    // msg!("file {}, line: {}", file!(), line!());

    // //整个池子的首次stake
    // let round_reward = pool_state.current_round_reward.view();
    // if pool_store.is_empty(){
    //    //reward 直接继承
    //    pool_store.push(Round { index: current_round_index,reward:round_reward, stake_amount:amount.view() });
    // //如果history_rounds的最后一个值等于current_round_index则说明，d当前轮次已经创建，直接更新即可
    // }else if pool_store.last().unwrap().0 == current_round_index {
    //     pool_store.last_mut().unwrap().2 +=  amount.view();
    // //如果history_rounds的最后一个值不等于current_round_index则说明，当前为这个轮次的第一个stake
    // }else if pool_store.last().unwrap().0 < current_round_index{
    //     //本地快照初始化继承上一轮的stake_amount值基础上增加当前用户质押数量
    //     let stake_amount =  pool_store.last().unwrap().2 + amount.view();
    //     pool_store.push(Round { index: current_round_index,reward:round_reward, stake_amount });
    // }else {
    //     unreachable!("")
    // }

    // msg!("file {}, line: {}", file!(), line!());

    // //update user state
    // require_gt!(
    //     MAX_USER_STAKE_TIMES,
    //     user_state.stakes.len(),
    //     StakeError::Unknown
    // );
    // user_state.user = ctx.accounts.user.key();
    // msg!("file {}, line: {}", file!(), line!());

    // //如果首次质押，则历史质押值为0
    // let default_stake = &UserStake::default();
    // let newest_stake_amount = user_state.stakes.last().unwrap_or(&default_stake).stake_amount + amount;
    // let user_stake = UserStake {
    //     round_index: current_round_index,
    //     stake_amount: newest_stake_amount,
    // };
    // msg!("file {}, line: {}", file!(), line!());

    // match user_state.stakes.last_mut() {
    //     // //当前轮次，首次,
    //     Some(stake) if stake.round_index < current_round_index  => {
    //         user_state.stakes.push(user_stake);
    //     }
    //     //当前轮次，多次质押
    //     Some(stake) if stake.round_index == current_round_index  => {
    //         stake.stake_amount = newest_stake_amount;
    //     } 
    //     //首次质押
    //     None => {
    //         user_state.stakes.push(user_stake);
    //     } 
    //     _ => {
    //         unreachable!("{} {}",line!(),file!());
    //     }
    // }
    // msg!("file {}, line: {}", file!(), line!());


    // let cpi_accounts = Transfer {
    //     from: ctx.accounts.user_vault.to_account_info(),
    //     to: ctx.accounts.pool_vault.to_account_info(),
    //     authority: ctx.accounts.user.to_account_info(),
    // };
    // msg!("file {}, line: {}", file!(), line!());
    // let cpi_program = ctx.accounts.token_program.to_account_info();
    // let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // token::transfer(cpi_ctx, amount)?;
    // msg!("file {}, line: {}", file!(), line!());

    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer=user,
        seeds=[user.key().as_ref(),pool_state.key().as_ref(), USER_STATE_SEED.as_bytes()], 
        bump,
        space = 8 + UserState::LEN
    )]
    pub user_state: Box<Account<'info, UserState>>,
    #[account(mut)]
    pub pool_state: Box<Account<'info, PoolState>>,
    #[account(mut)]
    pub pool_store: AccountLoader<'info, PoolStore>,
    #[account(mut,associated_token::mint = token_mint,associated_token::authority = user)]
    pub user_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, associated_token::mint = token_mint,
        associated_token::authority = pool_state)]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = token_mint.key() == pool_state.token_mint @ StakeError::NotMatchMint)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

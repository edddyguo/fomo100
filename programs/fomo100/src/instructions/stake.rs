use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::utils::{flatten_user_stake_snap, get_current_round_index};

pub fn handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require_gte!(
        amount,
        MINIMAL_STAKE_AMOUNT,
        StakeError::LessThanMinimalStakeAmount
    );

    let pool_state = &mut ctx.accounts.pool_state;
    let user_state = &mut ctx.accounts.user_state;

    let clock = Clock::get()?;
    let current_round = get_current_round_index(pool_state.created_at,clock.unix_timestamp,pool_state.round_period_secs); 
    let user_stakes_snap =  flatten_user_stake_snap(current_round,&user_state.stakes);
    //已解锁的禁止再质押
    //todo: 更多错误码
    if user_state.unlock_at.is_some(){
        Err( StakeError::Unknown)?;
    }

    //update pool's state
    require_gt!(
        ROUND_MAX,
        current_round,
        StakeError::HaveAlreadyFinished
    );
    pool_state.current_round_reward += amount;
    let mut history_rounds =  pool_state.history_rounds.as_mut_slice();
    history_rounds[current_round as usize].stake_amount += amount;

    
    //update user state
    require_gt!(
        MAX_USER_STAKE_TIMES,
        user_state.stakes.len() as u32,
        StakeError::Unknown
    );
    user_state.user = ctx.accounts.user.key();
  
    //本次质押后的最新值，user_stakes_snap是一定有值的，
    let newest_stake_amount = user_stakes_snap.last().unwrap() + amount;
    let user_stake = UserStake {
        round_index: current_round,
        stake_amount: newest_stake_amount,
    };
    match user_state.stakes.last_mut() {
        //当前轮次，首次
        Some(stake) if stake.round_index == current_round -1  => {
            user_state.stakes.push(user_stake);
        } 
        //当前轮次，多次质押
        Some(stake) if stake.round_index == current_round  => {
            stake.stake_amount = newest_stake_amount;
        } 
        //首次质押
        None => {
            user_state.stakes.push(user_stake);
        } 
        _ => {
            unreachable!("{} {}",line!(),file!());
        }
    }
    

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_vault.to_account_info(),
        to: ctx.accounts.pool_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    msg!("{} staked: {}", user_state.user, amount);

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
    pub user_state: Account<'info, UserState>,
    #[account(mut)]
    pub pool_state: Account<'info, PoolState>,
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

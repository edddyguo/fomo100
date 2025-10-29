use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use crate::utils::{flatten_pool_stake_snap, flatten_user_stake_snap, get_current_round_index};

pub fn handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require_gte!(
        amount,
        MINIMAL_STAKE_AMOUNT,
        StakeError::LessThanMinimalStakeAmount
    );

    let pool_state = &mut ctx.accounts.pool_state;
    let user_state = &mut ctx.accounts.user_state;

    let clock = Clock::get()?;
    let current_round_index = get_current_round_index(pool_state.created_at,clock.unix_timestamp,pool_state.round_period_secs); 
    //let user_stakes_snap =  flatten_user_stake_snap(current_round_index,&user_state.stakes);
    //已解锁的禁止再质押
    //todo: 更多错误码
    if user_state.unlock_at.is_some(){
        Err( StakeError::Unknown)?;
    }

    //update pool's state
    //todo: 临时注销后续在生产要明确告知结束了
    // require_gt!(
    //     ROUND_MAX,
    //     current_round_index,
    //     StakeError::HaveAlreadyFinished
    // );
    //\nindex out of bounds: the len is 0 but the index is 539",
    //pool_state.current_round_reward += amount;
    // let mut flatten_history_rounds = flatten_pool_stake_snap(current_round,&pool_state.history_rounds);
    // flatten_history_rounds[current_round as usize].stake_amount += amount;
    // pool_state.history_rounds.iter_mut().filter( = 
    //每轮次第一个质押需要创建pool的快照，后续的更新快照
    if let Some(round) = pool_state.history_rounds.iter_mut().find(|u| u.index == current_round_index) {
        round.stake_amount +=  amount;
    }else{
        //reward 直接继承
        let reward = pool_state.current_round_reward;
        //本地快照初始化继承上一轮的stake_amount值基础上增加当前用户质押数量
        let stake_amount =  pool_state.history_rounds.last().map_or(0, |x|x.stake_amount) + amount;
        pool_state.history_rounds.push(Round { index: current_round_index,reward, stake_amount });
    }
    
    //update user state
    require_gt!(
        MAX_USER_STAKE_TIMES,
        user_state.stakes.len() as u32,
        StakeError::Unknown
    );
    user_state.user = ctx.accounts.user.key();
  
    //如果首次质押，则历史质押值为0
    let default_stake = &UserStake::default();
    let newest_stake_amount = user_state.stakes.last().unwrap_or(&default_stake).stake_amount + amount;
    let user_stake = UserStake {
        round_index: current_round_index,
        stake_amount: newest_stake_amount,
    };
    match user_state.stakes.last_mut() {
        //当前轮次，首次,
        Some(stake) if stake.round_index < current_round_index  => {
            user_state.stakes.push(user_stake);
        } 
        //当前轮次，多次质押
        Some(stake) if stake.round_index == current_round_index  => {
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

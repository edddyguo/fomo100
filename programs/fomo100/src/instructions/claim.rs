use crate::utils::calculate_total_reward;
use crate::{errors::*, state::*, utils::get_current_round_index};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

//领取到目前为止的奖励
pub fn handler(ctx: Context<Claim>,    
    created_at: i64,
    round_period_secs: u32
) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;
    let pool_store = &mut ctx.accounts.pool_store.load_mut()?;
    let user_state = &mut ctx.accounts.user_state;

    let clock = Clock::get()?;
    let current_round_index = get_current_round_index(pool_state.created_at,clock.unix_timestamp,pool_state.round_period_secs);
    //已解锁的禁止再claim
    if user_state.unlock_at.is_some(){
        return Err(StakeError::AlreadyUnlocked)?;
    }

    if user_state.is_unstaked {
        Err(StakeError::AlreadyUnstake)?;
    }

    if pool_store.is_empty(){
        return Err(StakeError::PoolStoreIsEmpty)?;
    }

    msg!("current_round = {}",current_round_index);
    msg!("user_state = {:?}",user_state);
    msg!("pool_state = {:?}",pool_state);
    // msg!("pool_store = {:?}",pool_store);

    //当本轮次用户已经领取了，则归属奖励为0，则禁止再申领
    let reward_amount = calculate_total_reward(current_round_index,&pool_state,&pool_store,&user_state.stakes)?;
    msg!("reward_amount {}",reward_amount);
    if reward_amount == 0 {
        return Err(StakeError::RewardIsZero)?;
    }
    //update pool state
    pool_state.claimed_reward += reward_amount;

    //update user state
    //clear user's rounds before current round
    let newest_stake_amount = user_state.stakes.last().unwrap();
    let user_stake = UserStake {
        round_index: current_round_index,
        stake_amount: newest_stake_amount.stake_amount,
    };
    //重新从本轮次重新标记
    user_state.stakes = vec![user_stake];
    user_state.claimed_reward += reward_amount;

    //如果当前轮次还没有快照，需要冗余标记，以解决claim的所在轮次没有快照，计算不了的问题
    pool_store.create_or_update_snap(current_round_index, None,None);

    //4) 进行奖励发放
    let round_period_secs_bytes = pool_state.round_period_secs.to_be_bytes();
    let created_at_bytes = pool_state.created_at.to_be_bytes();

    let mint_key =  pool_state.token_mint.key();
    let signer = &[
        mint_key.as_ref(),
        created_at_bytes.as_ref(),
        round_period_secs_bytes.as_ref(),
        POOL_STATE_SEED.as_bytes(),
        &[ctx.bumps.pool_state],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.pool_vault.to_account_info(),
        to: ctx.accounts.user_ata.to_account_info(),
        authority: ctx.accounts.pool_state.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx.with_signer(&[signer]), reward_amount)?;

    msg!(
        "{} claimed: {},)",
        user_state.user,
        reward_amount
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(created_at:i64,round_period_secs: u32)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds=[user.key().as_ref(),pool_state.key().as_ref() , USER_STATE_SEED.as_bytes()], bump)]
    pub user_state: Account<'info, UserState>,
    #[account(mut, seeds=[token_mint.key().as_ref(),created_at.to_be_bytes().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STATE_SEED.as_bytes()], bump)]
    pub pool_state: Account<'info, PoolState>,
    #[account(mut)]
    pub pool_store: AccountLoader<'info, PoolStore>,
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::mint = token_mint,
        associated_token::authority = pool_state
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = token_mint.key() == pool_state.token_mint @ StakeError::NotMatchMint)]
    pub token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

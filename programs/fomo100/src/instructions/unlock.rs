use crate::{
    errors::*,
    state::*,
    utils::{AmountView, calculate_total_reward, get_current_round_index},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

pub fn handler(ctx: Context<Unlock>, created_at: i64, round_period_secs: u32) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let pool_store = &mut ctx.accounts.pool_store.load_mut()?;
    let pool_state = &mut ctx.accounts.pool_state;

    let clock = Clock::get()?;
    let current_round_index = get_current_round_index(
        pool_state.created_at,
        clock.unix_timestamp,
        pool_state.round_period_secs,
    );

    let token_scale = pool_state.token_scale;

    //如果已解锁，报错，禁止重复解锁
    if user_state.unlock_at.is_some(){
        Err( StakeError::AlreadyUnlocked)?;
    }

    //禁止在质押的轮次进行解锁
    //todo,也可以允许，只是质押轮次的奖励给零了
   let Some(user_last_stake) =  user_state.stakes.last() else {
      return Err( StakeError::StakeIsEmpty.into());
   };
   let user_stake_amount = user_last_stake.stake_amount;


    // 1） update pool state
    pool_state.unlocking_users += 1;
    pool_state.unlocking_stake_amount += user_stake_amount;

    //2） update user state
    //项目结束后，用户无需30天的等待期
    let unlock_at  = if pool_store.len() >= ROUND_MAX {
        clock.unix_timestamp
    }else {
        clock.unix_timestamp + pool_state.unlock_period_secs as i64
    };
    user_state.unlock_at = Some(unlock_at);

    //4) 如果有剩余的奖励尚未claim，则发给用户之前轮次的奖励，当前轮次的作废
    let reward_amount = calculate_total_reward(current_round_index,&pool_state,&pool_store,&user_state.stakes)?;
    if reward_amount != 0 {
        let round_period_secs_bytes = pool_state.round_period_secs.to_be_bytes();
        let created_at_bytes = pool_state.created_at.to_be_bytes();

        let token_mint_key = ctx.accounts.token_mint.key();
        let signer = &[
            token_mint_key.as_ref(),
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
            "{} unlock and claimed: {},)",
            user_state.user,
            reward_amount
        );
    }

    //5) update pool store,扣减总质押金额
    let last_round = pool_store.last().unwrap();
     msg!("last_round.round_index={} current_round_index={},",last_round.round_index , current_round_index);
    let current_stake_amount = last_round.stake_amount -  user_stake_amount.view(token_scale);
    pool_store.create_or_update_snap(current_round_index,None,Some(current_stake_amount));
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(created_at:i64,round_period_secs: u32)]
pub struct Unlock<'info> {
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

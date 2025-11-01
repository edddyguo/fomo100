use crate::{errors::*, state::*, utils::{calculate_total_reward, get_current_round_index, DAY1}};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

pub fn handler(ctx: Context<Unlock>,created_at:i64, round_period_secs: u32) -> Result<()> {
    // let user_state = &mut ctx.accounts.user_state;
    // let pool_store = &mut ctx.accounts.pool_store.load_init()?;
    // let pool_state = &mut ctx.accounts.pool_state;

    // let clock = Clock::get()?;
    // //如果已解锁，报错，禁止重复解锁
    // if user_state.unlock_at.is_some(){
    //     Err( StakeError::Unknown)?;
    // }
   
    // //update pool state
    // pool_state.unlocking_users += 1;
    // pool_state.unlocking_stake_amount += user_state.stakes.last().expect("when unlock,user must have already stake").stake_amount;
    // //update user state
    // user_state.unlock_at = Some(clock.unix_timestamp + DAY1 * UNLOCK_DAYS);

    // //step3: 如果有剩余的奖励尚未claim，则发给用户之前轮次的奖励，当前轮次的作废
    // let reward_amount = calculate_total_reward(&pool_store,&user_state.stakes)?;
    // if reward_amount != 0 {
    //     let round_period_secs_bytes = pool_state.round_period_secs.to_be_bytes();
    //     let created_at_bytes = pool_state.created_at.to_be_bytes();

    //     let token_mint_key = ctx.accounts.token_mint.key();
    //     let signer = &[
    //         token_mint_key.as_ref(),
    //         created_at_bytes.as_ref(),
    //         round_period_secs_bytes.as_ref(),
    //         POOL_STATE_SEED.as_bytes(),
    //         &[ctx.bumps.pool_state],
    //     ];
    
    //     let cpi_accounts = Transfer {
    //         from: ctx.accounts.pool_vault.to_account_info(),
    //         to: ctx.accounts.user_vault.to_account_info(),
    //         authority: ctx.accounts.pool_state.to_account_info(),
    //     };
    
    //     let cpi_program = ctx.accounts.token_program.to_account_info();
    //     let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    //     token::transfer(cpi_ctx.with_signer(&[signer]), reward_amount)?;
    
    //     msg!(
    //         "{} unlock and claimed: {},)",
    //         user_state.user,
    //         reward_amount
    //     );
    // }
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
    pub user_vault: InterfaceAccount<'info, TokenAccount>,
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



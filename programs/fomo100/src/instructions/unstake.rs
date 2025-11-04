use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Transfer},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

//todo: 保证逻辑上的更自然，unlock的时候会把用户的reward也顺带发给用户
pub fn handler(ctx: Context<Unstake>, created_at: i64, round_period_secs: u32) -> Result<()> {
    let user_state = &mut ctx.accounts.user_state;
    let pool_state = &mut ctx.accounts.pool_state;

    let clock = Clock::get()?;

    match user_state.unlock_at {
        Some(time) if clock.unix_timestamp > time => {
            msg!("start unstake");
        }
        Some(_) => {
            //未到解锁时间
            Err(StakeError::UnlockTimeNotArrived)?;
        }
        None => {
            //尚未解锁
            Err(StakeError::NotUnlock)?;
        }
    }

    if user_state.is_unstaked {
        Err(StakeError::AlreadyUnstake)?;
    }

    //todo:解除质押的尽量把用户的account也给回收掉，不刚需前端也许还有读状态
    let staked_amount = user_state
        .stakes
        .last()
        .expect("when unlock,user must have already stake")
        .stake_amount;

    //update user state
    user_state.is_unstaked = true;

    //update pool state
    //todo:
    //pool_state.claimed_reward += staked_amount;

    //step3: transfer stake amount
    let user_key = ctx.accounts.user.key();
    let pool_state_key =  pool_state.key();


   // seeds=[user.key().as_ref(),pool_state.key().as_ref(), USER_STATE_SEED.as_bytes()], 

    let signer = &[
        user_key.as_ref(),
        pool_state_key.as_ref(),
        USER_STATE_SEED.as_bytes(),
        &[ctx.bumps.user_state],
    ];

    let cpi_accounts = Transfer {
        from: ctx.accounts.user_vault.to_account_info(),
        to: ctx.accounts.user_ata.to_account_info(),
        authority: user_state.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx.with_signer(&[signer]), staked_amount)?;

    msg!("{} unstaked: {})", user_state.user, staked_amount,);

    Ok(())
}

#[derive(Accounts)]
#[instruction(created_at:i64,round_period_secs: u32)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds=[user.key().as_ref(),pool_state.key().as_ref() , USER_STATE_SEED.as_bytes()], bump)]
    pub user_state: Account<'info, UserState>,
    #[account(mut, seeds=[token_mint.key().as_ref(),created_at.to_be_bytes().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STATE_SEED.as_bytes()], bump)]
    pub pool_state: Account<'info, PoolState>,
    /// 用户质押的ata
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = user_state
    )]
    pub user_vault: InterfaceAccount<'info, TokenAccount>,
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

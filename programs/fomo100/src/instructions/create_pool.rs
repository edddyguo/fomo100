use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;

pub fn handler(ctx: Context<CreatePool>, round_period_secs: u32) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_state;

    let clock = Clock::get()?;

    pool_state.token_mint = ctx.accounts.token_mint.key();

    pool_state.round_period_secs = round_period_secs;

    pool_state.created_at = clock.unix_timestamp;

    pool_state.current_round_reward = 0;

    pool_state.history_rounds = Vec::new();

    pool_state.unlocking_users = 0;

    pool_state.unlocking_stake_amount = 0;

    pool_state.claimed_reward = 0;

    msg!("Initialize pool {}", pool_state.to_account_info().key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(round_period_secs: u32)]
pub struct CreatePool<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    //init pool state by ended_at
    #[account(init, payer=admin, seeds=[token_mint.key().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STATE_SEED.as_bytes()], bump, space=8 + PoolState::LEN)]
    pool_state: Account<'info, PoolState>,
    #[account(
        init,
        associated_token::mint = token_mint,
        associated_token::authority = pool_state,
        payer = admin,
    )]
    pool_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    token_mint: InterfaceAccount<'info, Mint>,
    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

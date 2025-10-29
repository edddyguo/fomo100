use crate::{
    errors::{unknown_error, StakeError},
    state::*,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_token::solana_program::sysvar::rewards;

const ADDED_SIZE_PER_TIME: usize = 10240;

pub fn handler(ctx: Context<ExpandPoolState>) -> Result<()> {
    let account_info = &ctx.accounts.pool_state;

    let current_size = account_info.data_len();

    // 每次扩容 <= 10KB
    let new_size = current_size + ADDED_SIZE_PER_TIME;
    msg!(
        "start expanded account from {} to {}",
        current_size,
        new_size
    );

    // ⚙️ 重新分配账户空间
    account_info.realloc(new_size, false)?;

    msg!(
        "achieve expanded account from {} to {}",
        current_size,
        new_size
    );
    Ok(())
}

#[derive(Accounts)]
pub struct ExpandPoolState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    ///  CHECK:
    #[account(mut, owner = crate::ID)]
    pub pool_state: AccountInfo<'info>,
}

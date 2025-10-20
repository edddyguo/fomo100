use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub fn handler(
    ctx: Context<SetAdmin>,
    new_admin: Option<Pubkey>,
    new_validator: Option<Pubkey>,
    new_treasurer: Option<Pubkey>,
) -> Result<()> {
    let admin_state = &mut ctx.accounts.admin_state;
    msg!("admin_state: {:?}", admin_state);

    //首次必须设置new_admin
    if let Some(new_admin) = new_admin {
        admin_state.admin = new_admin;
    } else if admin_state.admin.key() == Default::default() {
        return Err(MinterError::NoChange.into());
    }

    if let Some(new_validator) = new_validator {
        admin_state.validator = new_validator;
    }

    if let Some(new_treasurer) = new_treasurer {
        admin_state.treasurer = new_treasurer;
    }

    // if no change, return error
    if new_admin.is_none() && new_validator.is_none() && new_treasurer.is_none() {
        return Err(MinterError::NoChange.into());
    }

    msg!(
        "update admins successfully, current admins: {:?}",
        admin_state
    );
    Ok(())
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(
        init_if_needed,
        payer = admin,
        seeds=[ADMIN_STATE_SEED.as_bytes()], 
        bump,
        space = 8 + AdminState::LEN,
        constraint = admin_state.admin.key() == Default::default() || admin_state.admin.key() == admin.key() @ MinterError::InsufficientPermission
    )]
    admin_state: Account<'info, AdminState>,
    /// Admin signer
    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

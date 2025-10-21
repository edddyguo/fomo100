use crate::{errors::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

pub fn handler(
    ctx: Context<SetPrice>,
    sol_price: Option<u64>,
    settle_token_price: Option<u64>,
    settle_token: Option<Pubkey>,
) -> Result<()> {
    let collection_state = &mut ctx.accounts.collection_state;

    if let Some(sol_price) = sol_price {
        collection_state.sol_price = sol_price;
    }
    if let Some(price) = settle_token_price {
        collection_state.settle_token_price = price;
    }
    if let Some(settle_token) = settle_token {
        collection_state.settle_token = settle_token;
    }
    msg!(
        "update collection_state successfully,current collection_state: {:?}",
        collection_state
    );
    Ok(())
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    // Admin signer
    #[account(mut, address = admin_state.admin @  MinterError::InsufficientPermission)]
    pub admin: Signer<'info>,
    //collection mint
    #[account(mut)]
    collection_mint: InterfaceAccount<'info, Mint>,
    //
    #[account(
        mut,
        seeds=[COLLECTION_STATE_SEED.as_bytes(),collection_mint.key().as_ref()], 
        bump,
    )]
    collection_state: Account<'info, CollectionState>,

    #[account(
        mut, 
        seeds=[ADMIN_STATE_SEED.as_bytes()], 
        bump,
    )]
    admin_state: Account<'info, AdminState>,
}

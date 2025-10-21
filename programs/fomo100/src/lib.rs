pub mod constants;
pub mod errors;
pub mod interface;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use mpl_token_metadata::types::Collection;
use interface::create_collection::*;
use interface::init_airdrop::*;
use interface::mint_by_coin::*;
use interface::mint_by_sol::*;
use interface::set_admin::*;
use interface::set_price::*;
use std::str::FromStr;

declare_id!("4Th4Zf653GLACZ6yEEeharhv9JytsUYtPyXYAcA9freJ");

#[program]
pub mod fomo100 {
    use super::*;

    pub fn nft_mint_by_sol(ctx: Context<NftMintBySol>, id: u32) -> Result<()> {
        interface::mint_by_sol::handler(ctx, id)
    }

    pub fn nft_mint_by_coin(ctx: Context<NftMintByCoin>) -> Result<()> {
        interface::mint_by_coin::handler(ctx)
    }

    pub fn nft_create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        symbol: String,
        base_uri: String,
        sol_price: u64,
        settle_token_price: u64,
        settle_token: Pubkey,
    ) -> Result<()> {
        interface::create_collection::handler(
            ctx,
            name,
            symbol,
            base_uri,
            sol_price,
            settle_token_price,
            settle_token,
        )
    }

    pub fn set_admin(
        ctx: Context<SetAdmin>,
        new_admin: Option<Pubkey>,
        new_validator: Option<Pubkey>,
        new_treasurer: Option<Pubkey>,
    ) -> Result<()> {
        interface::set_admin::handler(ctx, new_admin, new_validator, new_treasurer)
    }

    pub fn set_price(
        ctx: Context<SetPrice>,
        sol_price: Option<u64>,
        settle_token_price: Option<u64>,
        settle_token: Option<Pubkey>,
    ) -> Result<()> {
        interface::set_price::handler(ctx, sol_price, settle_token_price, settle_token)
    }

    pub fn init_airdrop(ctx: Context<InitAirdrop>, amount: u32, sig: [u8; 64]) -> Result<()> {
        interface::init_airdrop::handler(ctx, amount, sig)
    }
}

#[derive(Accounts)]
pub struct SetData<'info> {
    #[account(mut)]
    pub puppet: Account<'info, Data>,
}

#[account]
pub struct Data {
    pub data: u64,
}

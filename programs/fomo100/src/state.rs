use anchor_lang::prelude::*;

pub const COLLECTION_AUTHORITY_SEED: &str = "collection_authority";
pub const COLLECTION_MINT_SEED: &str = "collection_mint";
pub const MINT_SEED: &str = "mint";
pub const COLLECTION_STATE_SEED: &str = "collection_state";
pub const USER_STATE_SEED: &str = "user";
pub const ADMIN_STATE_SEED: &str = "admin";

#[account]
#[derive(Debug)]
pub struct CollectionState {
    pub name: [u8; 32],
    pub symbol: [u8; 8],
    pub base_uri: [u8; 64],
    pub amount: u64,
    pub sol_price: u64,
    // protocol income
    pub sol_income: u64,
    pub settle_token: Pubkey,
    // last token_id == amount
    // 1_000_000_000 is 1 sol
    // 1_000_000_000 is 1 settle_token
    pub settle_token_price: u64,
    // protocol income
    pub settle_token_income: u64,
}

impl CollectionState {
    pub const LEN: usize = 32 + 8 + 64 + 32 + 8 + 8 + 8 + 8 + 8;
}

#[account]
#[derive(Debug)]
pub struct AdminState {
    // permission: set price,set new admin,create collection
    pub admin: Pubkey,
    // free mint validator
    pub validator: Pubkey,
    // receive protocol income
    pub treasurer: Pubkey,
}

impl AdminState {
    pub const LEN: usize = 32 + 32 + 32;
}

#[account]
#[derive(Debug)]
pub struct UserState {
    /// unclaimed airdrop number
    pub pending_airdrop_count: u32,
    /// already claimed number
    pub claimed_airdrop_count: u32,
    /// number of minted tokens by pay sol or usdt
    pub payed_count: u16,
}

impl UserState {
    pub const LEN: usize = 2 + 4 + 4;
}

use anchor_lang::prelude::*;

pub const MINIMAL_STAKE_AMOUNT: u64 = 1_000_000_000;

pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_STATE_SEED: &str = "pool_state";
pub const USER_VAULT_SEED: &str = "user_vault";
pub const USER_STATE_SEED: &str = "user_state";

#[account]
#[derive(Debug)]
pub struct UserState {
    pub user: Pubkey,
    pub times: u32,
    pub amount: u64,
    pub point: u128,
    pub apply_unlock_at: i64,
    pub is_claimed: bool,
}

impl UserState {
    pub const LEN: usize = 32 + 4 + 8 + 16 + 8 + 1;
}

#[account]
#[derive(Debug)]
pub struct PoolState {
    pub token_mint: Pubkey,
    pub unlock_period_days: u16,
    pub created_at: i64,
    pub total_users: u32,
    pub staking_users: u32,
    pub staking_amount: u64,
    pub staking_point: u128,
    pub unlocking_users: u32,
    pub unlocking_amount: u64,
    pub unlocking_point: u128,
    pub claimed_users: u32,
    pub claimed_amount: u64,
    pub claimed_point: u128,
}

impl PoolState {
    pub const LEN: usize = 32 + 2 + 8 + 4 + 4 + 8 + 16 + 4 + 8 + 16 + 4 + 8 + 16;
}

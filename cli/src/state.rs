use anchor_client::anchor_lang::Key;

use anyhow::Result;
use fomo100::state::PoolState;
use fomo100::state::UserState;
use fomo100::state::*;
use solana_sdk::signature::Keypair;

use std::rc::Rc;

use anchor_client::anchor_lang::prelude::Pubkey;

use anchor_client::Program;
#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};

pub trait State {
    fn user_state<T: Into<Pubkey> + Clone>(
        &self,
        collection_mint: &T,
        created_at: i64,
        round_period_secs: u32,
        user_pubkey: &T,
    ) -> Result<UserState>;
    fn user_states<T: Into<Pubkey> + Clone>(
        &self,
        collection_mints: &[T],
        user_pubkeys: &[T],
    ) -> Result<Vec<UserState>>;
    fn pool_state<T: Into<Pubkey> + Clone>(
        &self,
        token_mint: &T,
        created_at: i64,
        round_period_secs: u32,
    ) -> Result<PoolState>;

    fn pool_store<T: Into<Pubkey> + Clone>(
        &self,
        token_mint: &T,
        created_at: i64,
        round_period_secs: u32,
    ) -> Result<PoolStore>;
}

impl State for Program<Rc<Keypair>> {
    fn user_state<T: Into<Pubkey> + Clone>(
        &self,
        token_mint: &T,
        created_at: i64,
        round_period_secs: u32,
        user_pubkey: &T,
    ) -> Result<UserState> {
        let token_mint: Pubkey = token_mint.clone().into();
        let (pool_state_pda, _bump) = Pubkey::find_program_address(
            &[
                token_mint.key().as_ref(),
                created_at.to_be_bytes().as_ref(),
                round_period_secs.to_be_bytes().as_ref(),
                POOL_STATE_SEED.as_bytes(),
            ],
            &self.id(),
        );

        let user_pubkey: Pubkey = user_pubkey.clone().into();
        let (pda, _bump) = Pubkey::find_program_address(
            &[
                user_pubkey.key().as_ref(),
                pool_state_pda.key().as_ref(),
                USER_STATE_SEED.as_bytes(),
            ],
            &self.id(),
        );
        let collection_state = self.account::<UserState>(pda)?;
        println!("pool_state {:#?}", collection_state);
        Ok(collection_state)
    }
    //todo: 一次性拿全部,通过get_program_accounts_with_config
    fn user_states<T: Into<Pubkey> + Clone>(
        &self,
        collection_mints: &[T],
        user_pubkeys: &[T],
    ) -> Result<Vec<UserState>> {
        // let mut user_states = Vec::new();
        // for collection_mint in collection_mints {
        //     for user_pubkey in user_pubkeys {
        //         let user_state = self.user_state(collection_mint, user_pubkey)?;
        //         user_states.push(user_state);
        //     }
        // }
        // Ok(user_states)
        todo!()
    }

    //#[account(init, payer=admin, seeds=[token_mint.key().as_ref(),round_period_secs.to_be_bytes().as_ref(),POOL_STATE_SEED.as_bytes()], bump, space=8 + PoolState::LEN)]
    fn pool_state<T: Into<Pubkey> + Clone>(
        &self,
        token_mint: &T,
        created_at: i64,
        round_period_secs: u32,
    ) -> Result<PoolState> {
        let token_mint_pubkey: Pubkey = token_mint.clone().into();
        let (pda, _bump) = Pubkey::find_program_address(
            &[
                token_mint_pubkey.key().as_ref(),
                created_at.to_be_bytes().as_ref(),
                round_period_secs.to_be_bytes().as_ref(),
                POOL_STATE_SEED.as_bytes(),
            ],
            &self.id(),
        );
        let collection_state = self.account::<PoolState>(pda)?;
        println!("pool_state {:#?}", collection_state);
        Ok(collection_state)
    }

    fn pool_store<T: Into<Pubkey> + Clone>(
        &self,
        token_mint: &T,
        created_at: i64,
        round_period_secs: u32,
    ) -> Result<PoolStore> {
        let token_mint_pubkey: Pubkey = token_mint.clone().into();
        let (pda, _bump) = Pubkey::find_program_address(
            &[
                token_mint_pubkey.key().as_ref(),
                created_at.to_be_bytes().as_ref(),
                round_period_secs.to_be_bytes().as_ref(),
                POOL_STORE_SEED.as_bytes(),
            ],
            &self.id(),
        );
        let collection_state = self.account::<PoolStore>(pda)?;
        println!("pool_state {:?}", collection_state);
        Ok(collection_state)
    }
}

#[cfg(test)]
mod tests {
    // 导入测试模块
    use super::*;
    use anyhow::Ok;
    use fomo100::utils::calculate_total_reward;

    #[test]
    fn test_calculate_total_reward3() -> anyhow::Result<()> {
        let mut pool_store = PoolStore {
            reward_indexes: std::array::from_fn(|_| u8::MAX),
            round_indexes: std::array::from_fn(|_| u16::MAX),
            stake_amounts: std::array::from_fn(|_| u32::MAX),
            len: 0,
        };
        let round_indexes: Vec<u16> = vec![
            4, 6, 7, 349, 361, 363, 364, 365, 371, 372, 391, 645, 879, 898, 914, 920, 939, 941,
            944, 947, 948, 1245,
            // 1246,
        ];
        let reward_indexes = vec![
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            // 2,
        ];

        let stake_amounts = vec![
            2, 2, 0, 1, 3, 3, 3, 2, 5, 338, 338, 338, 339, 341, 341, 341, 341, 341, 341, 341, 342,
            343, //343,
        ];
        pool_store.len = round_indexes.len() as u32;
        pool_store.round_indexes[0..round_indexes.len()].copy_from_slice(&round_indexes);
        pool_store.reward_indexes[0..round_indexes.len()].copy_from_slice(&reward_indexes);
        pool_store.stake_amounts[0..round_indexes.len()].copy_from_slice(&stake_amounts);

        let pool_state = PoolState {
            admin: "7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf".try_into()?,
            token_mint: "CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h".try_into()?,
            token_scale: 1000000000,
            min_stake_amount: 1000000000,
            round_period_secs: 302,
            unlock_period_secs: 300,
            unlocking_stake_amount: 3000000000,
            claimed_reward: 91846294586,
            created_at: 1762313139,
            current_round_reward: 123456789,
            unlocking_users: 2,
            history_round_rewards: vec![100, 44000, 123456789],
        };

        let user_stakes = vec![UserStake {
            round_index: 947,
            stake_amount: 339000000000,
        }];
        let current_index = 1246;
        let res = calculate_total_reward(current_index, &pool_state, &pool_store, &user_stakes)?;
        assert_eq!(res, 36589778459);
        Ok(())
    }

    #[test]
    fn test_calculate_total_reward4() -> anyhow::Result<()> {
        let mut pool_store = PoolStore {
            reward_indexes: std::array::from_fn(|_| u8::MAX),
            round_indexes: std::array::from_fn(|_| u16::MAX),
            stake_amounts: std::array::from_fn(|_| u32::MAX),
            len: 0,
        };
        let round_indexes: Vec<u16> = vec![
            4, 6, 7, 349, 361, 363, 364, 365, 371, 372, 391, 645, 879, 898, 914, 920, 939, 941,
            944, 947, 948, 1245, 1246, 1431,
        ];
        let reward_indexes = vec![
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
        ];

        let stake_amounts = vec![
            2, 2, 0, 1, 3, 3, 3, 2, 5, 338, 338, 338, 339, 341, 341, 341, 341, 341, 341, 341, 342,
            343, 343, 343,
        ];
        pool_store.len = round_indexes.len() as u32;
        pool_store.round_indexes[0..round_indexes.len()].copy_from_slice(&round_indexes);
        pool_store.reward_indexes[0..round_indexes.len()].copy_from_slice(&reward_indexes);
        pool_store.stake_amounts[0..round_indexes.len()].copy_from_slice(&stake_amounts);

        let pool_state = PoolState {
            admin: "7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf".try_into()?,
            token_mint: "CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h".try_into()?,
            token_scale: 1000000000,
            min_stake_amount: 1000000000,
            round_period_secs: 302,
            unlock_period_secs: 300,
            unlocking_stake_amount: 3000000000,
            claimed_reward: 91846294586,
            created_at: 1762313139,
            current_round_reward: 123456789,
            unlocking_users: 2,
            history_round_rewards: vec![100, 44000, 123456789],
        };

        let user_stakes = vec![UserStake {
            round_index: 1431,
            stake_amount: 2000000000,
        }];
        let current_index = 1452;
        let res = calculate_total_reward(current_index, &pool_state, &pool_store, &user_stakes)?;
        println!("res {}", res);
        assert_eq!(res, 15117144);
        Ok(())
    }

    #[test]
    fn test_calculate_total_reward5() -> anyhow::Result<()> {
        let mut pool_store = PoolStore {
            reward_indexes: std::array::from_fn(|_| u8::MAX),
            round_indexes: std::array::from_fn(|_| u16::MAX),
            stake_amounts: std::array::from_fn(|_| u32::MAX),
            len: 0,
        };
        let round_indexes: Vec<u16> = vec![9, 10];
        let reward_indexes = vec![0, 0];

        let stake_amounts = vec![4, 5];
        pool_store.len = round_indexes.len() as u32;
        pool_store.round_indexes[0..round_indexes.len()].copy_from_slice(&round_indexes);
        pool_store.reward_indexes[0..round_indexes.len()].copy_from_slice(&reward_indexes);
        pool_store.stake_amounts[0..round_indexes.len()].copy_from_slice(&stake_amounts);

        let pool_state = PoolState {
            admin: "7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf".try_into()?,
            token_mint: "DvLeK1ff2pnVKn1XEMUvVH2wwj9rjtUzPTPLqN8DRtpk".try_into()?,
            token_scale: 1000000,
            min_stake_amount: 1000000,
            round_period_secs: 120,
            unlock_period_secs: 60,
            unlocking_stake_amount: 0,
            claimed_reward: 91846294586,
            created_at: 1762827326,
            current_round_reward: 1000000,
            unlocking_users: 2,
            history_round_rewards: vec![1000000],
        };

        let user_stakes = vec![
            UserStake {
                round_index: 9,
                stake_amount: 4000000,
            },
            UserStake {
                round_index: 10,
                stake_amount: 5000000,
            },
        ];
        let current_index = 10;
        let res = calculate_total_reward(current_index, &pool_state, &pool_store, &user_stakes)?;
        assert_eq!(res, 1_000_000);
        Ok(())
    }
}

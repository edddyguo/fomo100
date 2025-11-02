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
        println!("pool_state {:?}", collection_state);
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
        println!("pool_state {:?}", collection_state);
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

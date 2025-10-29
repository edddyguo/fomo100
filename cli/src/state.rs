use anchor_client::anchor_lang::accounts::account::Account;
use anchor_client::anchor_lang::Key;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::Signer;

use anchor_client::Client;

use anyhow::Result;
use fomo100::state::PoolState;
use fomo100::state::UserState;
use fomo100::state::USER_STATE_SEED;
use fomo100::state::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use spl_associated_token_account::get_associated_token_address;

use std::rc::Rc;
use std::str::FromStr;

use anchor_client::anchor_lang::prelude::Pubkey;
use solana_sdk::account::ReadableAccount;

use crate::SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID;
use crate::SPL_PROGRAM_ID;

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
        round_period_secs: u32,
    ) -> Result<PoolState>;
}

impl State for Program<Rc<Keypair>> {
    fn user_state<T: Into<Pubkey> + Clone>(
        &self,
        collection_mint: &T,
        user_pubkey: &T,
    ) -> Result<UserState> {
        // let user_pubkey: Pubkey = user_pubkey.clone().into();
        // let collection_mint_pubkey: Pubkey = collection_mint.clone().into();
        // let (pda, _bump) = Pubkey::find_program_address(
        //     &[
        //         USER_STATE_SEED.as_bytes(),
        //         collection_mint_pubkey.as_ref(),
        //         user_pubkey.as_ref(),
        //     ],
        //     &self.id(),
        // );
        // let user_state = self.account::<UserState>(pda)?;
        // Ok(user_state)
        todo!()
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
        round_period_secs: u32,
    ) -> Result<PoolState> {
        let token_mint_pubkey: Pubkey = token_mint.clone().into();
        let (pda, _bump) = Pubkey::find_program_address(
            &[
                token_mint_pubkey.key().as_ref(),
                round_period_secs.to_be_bytes().as_ref(),
                POOL_STATE_SEED.as_bytes(),
            ],
            &self.id(),
        );
        let collection_state = self.account::<PoolState>(pda)?;
        println!("pool_state {:?}", collection_state);
        Ok(collection_state)
    }
}

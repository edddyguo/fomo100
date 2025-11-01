pub mod general;
pub mod instructions;
pub mod math;
pub mod service;
pub mod state;
pub mod utils;

use crate::state::State;
use crate::utils::*;
use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::{Client, Cluster};
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use fomo100::constants::INIT_AIRDROP_SIGN_PREFIX;
use math::coin_amount::display2raw;
use service::SetNftClaimSigRequest;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::thread::{self, sleep};
use std::time::Duration;
use tokio::runtime::Runtime;
use utils::{current_date, get_lamport_balance};

#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};
#[derive(Parser, Debug)]
pub struct ExpandPoolState {
    #[clap(long)]
    pub program_id: String,
    /// 待扩展的账号，pool_state_pda
    #[clap(long)]
    pub account: String,
    /// 扩展的次数，每次10k
    #[clap(long)]
    pub times: u32,
}

#[derive(Parser, Debug)]
pub struct CreatePoolArgs {
    #[clap(long)]
    pub program_id: String,
    #[clap(long)]
    pub token_mint: String,
    #[clap(long)]
    pub created_at: i64,
    #[clap(long)]
    pub round_period_secs: u32,
}

#[derive(Parser, Debug)]
pub struct StakeArgs {
    #[clap(long)]
    pub program_id: String,
    #[clap(long)]
    pub token_mint: String,
    #[clap(long)]
    pub created_at: i64,
    #[clap(long)]
    pub round_period_secs: u32,
    #[clap(long)]
    pub stake_amount: u64,
}

#[derive(Parser, Debug)]
pub struct SetAdminArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub new_admin: Option<String>,
    #[clap(long)]
    pub new_validator: Option<String>,
    #[clap(long)]
    pub new_treasurer: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PoolStateArgs {
    #[clap(long)]
    pub program_id: String,
    #[clap(long)]
    pub token_mint: String,
    #[clap(long)]
    pub created_at: i64,
    #[clap(long)]
    pub round_period_secs: u32,
}

#[derive(Parser, Debug)]
pub struct SetPriceArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub collection_name: String,
    #[clap(long)]
    pub sol_price: Option<u64>,
    #[clap(long)]
    pub settle_token_price: Option<u64>,
    #[clap(long)]
    pub settle_token: Option<String>,
}

#[derive(Parser, Debug)]
pub struct InitAirdropArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub collection_name: String,
    #[clap(long)]
    pub init_amount: u32,
    #[clap(long)]
    pub init_sig: String,
    #[clap(long)]
    pub init_instruction_data: String,
}

#[derive(Parser, Debug)]
pub struct SignAirdropArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub collection_name: String,
    #[clap(long)]
    pub pubkey: String,
    #[clap(long)]
    pub amount: u32,
}

#[derive(Parser, Debug)]
pub struct UpdateNftSigListArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub core_collection_name: String,
    #[clap(long)]
    pub elite_collection_name: String,
}

#[derive(Parser, Debug)]
pub struct CheckAndUpdateNftSigListArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub core_collection_name: String,
    #[clap(long)]
    pub elite_collection_name: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    ExpandPoolState(ExpandPoolState),
    CreatePool(CreatePoolArgs),
    Stake(StakeArgs),
    SetAdmin(SetAdminArgs),
    PoolState(PoolStateArgs),
    SetPrice(SetPriceArgs),
    InitAirdrop(InitAirdropArgs),
    SignAirdrop(SignAirdropArgs),
    UpdateNftSigList(UpdateNftSigListArgs),
    CheckAndUpdateNftSigList(CheckAndUpdateNftSigListArgs),
}

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(long)]
    pub rpc_url: String,
    #[clap(long)]
    pub prikey: String,
    #[clap(subcommand)]
    pub subcommand: Commands,
}
//dev: new_spl
//const token_mint: &'static str = "2gwcxasrZpc4jC3NUAxD3dEKzh5v5ZoGwBJPRJ57RoXp";
//主
//const token_mint: &'static str = "Hax9LTgsQkze1YFychnBLtFH8gYbQKtKfWKKg2SP6gdD";
//dev:old_token,kin
//const token_mint: &'static str = "Ds1bpF3ZWUmg8rwWPszC635rTVBs7brpJYxZm3Jr2tZN";
static mut TOKEN_MINT: Option<Pubkey> = None;

/***
let url = Cluster::Custom(
    "https://api.mainnet-beta.solana.com".to_string(),
    "wss://api.mainnet-beta.solana.com/".to_string(),
);
let url = Cluster::Custom(
    "https://smart-ancient-general.solana-mainnet.quiknode.pro/d6a1ac1e03719deff67eef6ba8d02f8ac08a530a/".to_string(),
    "wss://api.mainnet-beta.solana.com/".to_string(),
);
let url = Cluster::Custom(
    "https://api.devnet.solana.com".to_string(),
    "wss://api.devnet.solana.com/".to_string(),
);
**/

static mut RPC: Option<String> = None;
static mut PRIKEY: Option<String> = None;

const fomo100: &str = "79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu";
const K_COIN: &'static str = "5d1i4wKHhGXXkdZB22iKD1SqU6pkBeTCwFEMqo7xy39h";
const SPL_PROGRAM_ID: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
//一键生成NFT的合约,待废弃
const NFT_MINT_CONTRACT: &'static str = "9HiRJw3dYo2MV9B1WrqFfoNjWRPS19mjVDCPqAxuMPfb";
const SENDER: &'static str = "9hUYW9s2c98GfjZb6JvW62BYEt3ryxGmeMBkhgSqmZtW";
const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: &'static str =
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const SYSTEM_PROGRAM_ID: &'static str = "11111111111111111111111111111111";
const SYSTEM_RENT_ID: &'static str = "SysvarRent111111111111111111111111111111111";
const MPL_TOKEN_METADATA_ACCOUNT: &'static str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
const MEM_COLLECTION_MINT: &'static str = "8zKSXBACKpaKvgDCYdDwpJGTVDSBCtAgucJpmR7gAyx5";

fn main() -> Result<()> {
    /***
    let url = Cluster::Custom(
        "https://api.mainnet-beta.solana.com".to_string(),
        "wss://api.mainnet-beta.solana.com/".to_string(),
    );
    let url = Cluster::Custom(
        "https://smart-ancient-general.solana-mainnet.quiknode.pro/d6a1ac1e03719deff67eef6ba8d02f8ac08a530a/".to_string(),
        "wss://api.mainnet-beta.solana.com/".to_string(),
    );
    let url = Cluster::Custom(
        "https://api.devnet.solana.com".to_string(),
        "wss://api.devnet.solana.com/".to_string(),
    );
    **/

    // Client.
    let Opts {
        rpc_url,
        prikey,
        subcommand,
    } = Opts::parse();

    let payer = Keypair::from_base58_string(&prikey);
    let cluster = Cluster::Custom(rpc_url.clone(), "".to_string());
    unsafe {
        RPC = Some(rpc_url);
    }
    let client = Client::new_with_options(cluster, Rc::new(payer), CommitmentConfig::confirmed());
    match subcommand {
        Commands::CreatePool(args) => {
            let program = client.program(Pubkey::from_str(&args.program_id)?)?;
            instructions::create_pool(
                &program,
                args.token_mint.as_str(),
                args.created_at,
                args.round_period_secs,
            )?;
        }
        Commands::ExpandPoolState(args) => {
            let program = client.program(Pubkey::from_str(&args.program_id)?)?;
            for _ in 0..args.times {
                instructions::expand_pool_state(&program, args.account.as_str())?;
                sleep(Duration::from_secs(5));
            }
        }
        Commands::Stake(args) => {
            let program = client.program(Pubkey::from_str(&args.program_id)?)?;
            instructions::stake(
                &program,
                args.token_mint.as_str(),
                args.created_at,
                args.round_period_secs,
                args.stake_amount,
            )?;
        }
        Commands::SetAdmin(args) => {
            todo!()
        }
        Commands::PoolState(args) => {
            let program = client.program(Pubkey::from_str(&args.program_id)?)?;
            let token_mint: Pubkey = args.token_mint.as_str().try_into().ok().unwrap();
            program.pool_state(&token_mint, args.created_at, args.round_period_secs)?;
            program.pool_store(&token_mint, args.created_at, args.round_period_secs)?;
        }
        Commands::SetPrice(args) => {
            todo!()
        }
        Commands::InitAirdrop(args) => {
            todo!()
        }
        Commands::SignAirdrop(args) => {
            todo!()
        }

        Commands::UpdateNftSigList(args) => {
            todo!()
        }
        Commands::CheckAndUpdateNftSigList(args) => {
            todo!()
        }
    }
    Ok(())
}

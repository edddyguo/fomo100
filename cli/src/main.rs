pub mod general;
pub mod math;
pub mod nft;
pub mod state;
pub mod utils;
pub mod service;

use crate::utils::*;
use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::{Client, Cluster};
use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use math::coin_amount::display2raw;
use service::SetNftClaimSigRequest;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use state::TotalNft;
use fomo100::state::COLLECTION_MINT_SEED;
use tokio::runtime::Runtime;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use fomo100::constants::INIT_AIRDROP_SIGN_PREFIX;
use utils::{current_date, get_lamport_balance};
use crate::state::State;

#[cfg(feature = "serde-feature")]
use {
    serde::{Deserialize, Serialize},
    serde_with::{As, DisplayFromStr},
};
#[derive(Parser, Debug)]
pub struct TransferArgs {
    /// 待处理的账户列表
    #[clap(long)]
    pub account_list: String,
    /// 经验值为5
    #[clap(long)]
    pub batch_size: u32,
    /// 处理结果文件
    #[clap(long)]
    pub account_result: String,
}

#[derive(Parser, Debug)]
pub struct MintArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub collection_name: String,
    #[clap(long)]
    pub pay_sol: bool,
    #[clap(long)]
    pub init_amount: u32,
    #[clap(long)]
    pub init_sig: String,
    #[clap(long)]
    pub init_instruction_data: String,
}

#[derive(Parser, Debug)]
pub struct CreateCollectionArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub name: String,
    #[clap(long)]
    pub symbol: String,
    #[clap(long)]
    pub uri: String,
    #[clap(long)]
    pub sol_price: Option<u64>,
    #[clap(long)]
    pub settle_token_price: Option<u64>,
    #[clap(long)]
    pub settle_token: String,
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
pub struct GetAdminArgs {
    #[clap(long)]
    pub minter_program_id: String,
    #[clap(long)]
    pub elite_collection_name: String,
    #[clap(long)]
    pub core_collection_name: String,
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
    Transfer(TransferArgs),
    Mint(MintArgs),
    CreateCollection(CreateCollectionArgs),
    SetAdmin(SetAdminArgs),
    GetAdmin(GetAdminArgs),
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
    let client = Client::new_with_options(cluster, Rc::new(payer), CommitmentConfig::confirmed());

    // unsafe {
    //     DOJO_COIN = Some(token_mint_address);
    //     DOJO_STAKING_CONTRACT_ID = Some(program_id);
    // }

    match subcommand {
        Commands::Mint(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::init_airdrop_and_mint(&program, args.collection_name, args.pay_sol, args.init_amount, args.init_sig, args.init_instruction_data)?;
        }
        Commands::Transfer(args) => {
            todo!()
        }
        Commands::CreateCollection(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::mint_collection(
                &program,
                args.name,
                args.symbol,
                args.uri,
                args.sol_price,
                args.settle_token_price,
                args.settle_token,
            )?;
        }
        Commands::SetAdmin(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::set_admin(
                &program,
                args.new_admin,
                args.new_validator,
                args.new_treasurer,
            )?;
        }
        Commands::GetAdmin(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            let admin_info = nft::get_admin(&program, args.elite_collection_name, args.core_collection_name)?;
            println!("admin_info: {:?}", admin_info);
        }
        Commands::SetPrice(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::set_price(
                &program,
                args.collection_name,
                args.sol_price,
                args.settle_token_price,
                args.settle_token,
            )?;
        }
        Commands::InitAirdrop(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::init_airdrop(&program, args.collection_name, args.init_amount, args.init_sig, args.init_instruction_data)?;
        }
        Commands::SignAirdrop(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            nft::sign_airdrop(&program, &prikey, args.collection_name, args.pubkey, args.amount)?;
        }

        Commands::UpdateNftSigList(args) => {
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            let pending_sig_list = service::get_claimed_list();
            let set_data: Vec<SetNftClaimSigRequest> = pending_sig_list.into_iter().map(|x| {
                let nft_sig = nft::sign_airdrop(&program, &prikey, args.core_collection_name.clone(),x.address.clone(), x.nft_amount).unwrap();
                let nft_elite_sig = nft::sign_airdrop(&program, &prikey, args.elite_collection_name.clone(),x.address.clone(), x.nft_premium_amount).unwrap();
                SetNftClaimSigRequest{
                    address: x.address,
                    nft_sig: nft_sig,
                    nft_premium_sig: nft_elite_sig,
                }
            }).collect();
            service::set_claimed_sig(set_data);
            let res =  service::get_claimed_list();
            println!("get_claimed_list_res: {:?}", res); 
        }
        Commands::CheckAndUpdateNftSigList(args) => {
           //心跳进程发飞书
           //检查上次nft的更新时间，是否等于当前的nft的更新时间，如果等于，则不更新，否则更新
            let program = client.program(Pubkey::from_str(&args.minter_program_id)?)?;
            loop {
                let claim_states_off_chain = service::get_claimed_list();
                let mut need_update_sig_list = Vec::new();
                for state_off_chain in claim_states_off_chain.iter() {
                    let current_nft_core_sig = nft::sign_airdrop(&program, &prikey, args.core_collection_name.clone(), state_off_chain.address.clone(), state_off_chain.nft_amount).unwrap();
                    let current_nft_elite_sig = nft::sign_airdrop(&program, &prikey, args.elite_collection_name.clone(), state_off_chain.address.clone(), state_off_chain.nft_premium_amount).unwrap();
                    //如果签名不一致，则说明首次注入签名，或者有新的空投，需要更新签名
                    if state_off_chain.nft_sig != current_nft_core_sig || state_off_chain.nft_premium_sig != current_nft_elite_sig {
                        need_update_sig_list.push(SetNftClaimSigRequest {
                            address: state_off_chain.address.clone(),
                            nft_sig: current_nft_core_sig,
                            nft_premium_sig: current_nft_elite_sig,
                        });
                    }
                }
             
                let msg: String = format!("{},find new nft id need sign: {:#?}",current_date(), need_update_sig_list);
                if !need_update_sig_list.is_empty() {
                    service::set_claimed_sig(need_update_sig_list);
                }
                println!("{:?}",msg);
                notify_lark(&msg)?;
                thread::sleep(Duration::from_secs(60));
            }
        }
    }
    Ok(())
}

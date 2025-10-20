//todo: some solana api
use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::ClientError;
use anyhow::Result;
use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::create_associated_token_account;
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer;
use spl_token::state::Account;
use spl_token::state::Account as TokenAccount;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::str::FromStr;

pub fn spl_transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, amount: u64) -> Result<String> {
    let mint_pubkey = unsafe { crate::TOKEN_MINT.unwrap() };
    let rpc_url = unsafe { crate::RPC.as_deref().unwrap() };
    let prikey = unsafe { crate::PRIKEY.as_deref().unwrap() };
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let payer = Keypair::from_base58_string(prikey); //Keypair::generate(&mut rand::thread_rng());
                                                     // 获取 token 账户地址
    let from_token_account =
        spl_associated_token_account::get_associated_token_address(from_pubkey, &mint_pubkey);
    let to_token_account =
        spl_associated_token_account::get_associated_token_address(to_pubkey, &mint_pubkey);
    let to_account_info = client.get_account(&to_token_account);
    let mut instructions = vec![];

    // 如果目标 token 账户不存在，则创建它
    if to_account_info.is_err() {
        let create_account_instruction =
            create_associated_token_account(from_pubkey, to_pubkey, &mint_pubkey);
        instructions.push(create_account_instruction);
    }

    // 创建转账指令
    let ins = transfer(
        &spl_token::id(),
        &from_token_account,
        &to_token_account,
        from_pubkey,
        &[],
        amount,
    )?;
    instructions.push(ins);

    // 创建交易
    let mut transaction = Transaction::new_with_payer(&instructions, Some(from_pubkey));

    // 签名交易
    let latest_block_hash = client.get_latest_blockhash()?;
    transaction.sign(&[&payer], latest_block_hash);
    println!("Transaction signature: {:?}", transaction.signatures);
    // 发送交易
    let signature = client.send_and_confirm_transaction(&transaction)?;
    Ok(signature.to_string())
}

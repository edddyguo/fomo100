use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::anchor_lang::solana_program::program_pack::Pack;
use anchor_client::anchor_lang::Key;
use anchor_client::solana_sdk::signature::read_keypair_file;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::{Client, Cluster};
use anyhow::Result;
use borsh::BorshDeserialize;
use mpl_token_metadata::types::Collection;
use solana_client::nonce_utils::get_account;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::ReadableAccount;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::ed25519_instruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::timing::timestamp;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{Account, AccountState};
use std::fs::Metadata;
use std::rc::Rc;
use std::str::FromStr;
use std::u64;
use fomo100::accounts as fomo100_accounts;
use fomo100::instruction as fomo100_instructions;
use fomo100::state::ADMIN_STATE_SEED;
use fomo100::state::COLLECTION_STATE_SEED;
use fomo100::state::*;
use serde_json::json;

use crate::state::State;
use crate::utils::{find_master_edition_pda, find_metadata_pda};
use crate::{
    MPL_TOKEN_METADATA_ACCOUNT, SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SPL_PROGRAM_ID,
    SYSTEM_PROGRAM_ID, SYSTEM_RENT_ID,
};

pub fn mint_collection(
    program: &anchor_client::Program<Rc<Keypair>>,
    name: String,
    symbol: String,
    base_uri: String,
    sol_price: Option<u64>,
    settle_token_price: Option<u64>,
    settle_token: String,
) -> Result<Pubkey> {
    //default is MAX cong
    println!("file: {}, line!{}",file!(),line!());
    let sol_price = sol_price.unwrap_or(u64::MAX);
    let settle_token_price = settle_token_price.unwrap_or(u64::MAX);
    println!("file: {}, line!{}",file!(),line!());
    let payer_key = program.payer();
    // Derive the collection authority using the same seed logic
    let (collection_authority, _) =
        Pubkey::find_program_address(&[COLLECTION_AUTHORITY_SEED.as_bytes()], &program.id());
        println!("file: {}, line!{}",file!(),line!());
    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), name.as_bytes()],
        &program.id(),
    );
    println!("file: {}, line!{}",file!(),line!());
    let collection_token_account =
        get_associated_token_address(&collection_authority, &collection_mint);

    let (admin_pda, _bump) =
        Pubkey::find_program_address(&[ADMIN_STATE_SEED.as_bytes()], &program.id());
    let (collection_state, _bump) = Pubkey::find_program_address(
        &[COLLECTION_STATE_SEED.as_bytes(), collection_mint.as_ref()],
        &program.id(),
    );
    println!("file: {}, line!{}",file!(),line!());

    let collection_metadata = find_metadata_pda(&collection_mint);
    let collection_master_edition = find_master_edition_pda(&collection_mint);

    //print all the accounts
    println!("collection mint key {}", collection_mint.to_string());
    println!("collection_metadata: {}", collection_metadata.to_string());
    println!(
        "collection_master_edition: {}",
        collection_master_edition.to_string()
    );
    println!(
        "collection_token_account: {}",
        collection_token_account.to_string()
    );
    println!("collection_authority: {}", collection_authority.to_string());

    //panic!("stop here");
    let mint_build = program
        .request()
        .accounts(fomo100_accounts::CreateCollection {
            admin: payer_key,
            collection_mint: collection_mint,
            collection_metadata: collection_metadata,
            collection_master_edition: collection_master_edition,
            collection_token_account: collection_token_account,
            collection_authority: collection_authority,
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID)
                .unwrap(),
            metadata_program: Pubkey::from_str(MPL_TOKEN_METADATA_ACCOUNT).unwrap(),
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
            rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
            admin_state: admin_pda,
            collection_state: collection_state,
        })
        .args(fomo100_instructions::NftCreateCollection {
            name: name,
            symbol: symbol,
            base_uri: base_uri,
            sol_price,
            settle_token_price,
            settle_token: Pubkey::from_str(&settle_token)?,
        });

    println!(
        "mint_build: {:?}",
        mint_build.instructions()?.first().unwrap().to_owned()
    );
    let mint_res = program
        .request()
        .instruction(mint_build.instructions()?.first().unwrap().to_owned())
        .send();

    println!("mint_res: {:?}", mint_res);
    println!("collection mint key {}", collection_mint.to_string());
    let collection_state = program.collection_state(&collection_mint)?;
    println!("collection_state: {:?}", collection_state);
    Ok(collection_mint)
}

pub fn set_admin(
    program: &anchor_client::Program<Rc<Keypair>>,
    new_admin: Option<String>,
    validator: Option<String>,
    treasurer: Option<String>,
) -> Result<()> {
    let payer_key = program.payer();
    println!("payer_key: {}", payer_key.to_string());
    let new_admin = new_admin.map(|s| Pubkey::from_str(&s).unwrap());
    let new_validator = validator.map(|s| Pubkey::from_str(&s).unwrap());
    let new_treasurer = treasurer.map(|s| Pubkey::from_str(&s).unwrap());

    let (admin_pda, _bump) =
        Pubkey::find_program_address(&[ADMIN_STATE_SEED.as_bytes()], &program.id());

    //panic!("stop here");
    let mint_build = program
        .request()
        .accounts(fomo100_accounts::SetAdmin {
            admin_state: admin_pda,
            admin: payer_key,
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID)?,
        })
        .args(fomo100_instructions::SetAdmin {
            new_admin: new_admin,
            new_validator: new_validator,
            new_treasurer: new_treasurer,
        });

    println!(
        "mint_build: {:?}",
        mint_build.instructions()?.first().unwrap().to_owned()
    );
    let res = program
        .request()
        .instruction(mint_build.instructions()?.first().unwrap().to_owned())
        .send();

    println!("mint_res: {:?}", res);
    let mint_res = res?;
    println!("pda_state_address: {}", admin_pda.to_string());
    let admin_state = program.admin_state()?;
    println!("admin_state: {:?}", admin_state);
    Ok(())
}

pub fn set_price(
    program: &anchor_client::Program<Rc<Keypair>>,
    //collection_mint: Pubkey,
    name: String,
    sol_price: Option<u64>,
    settle_token_price: Option<u64>,
    settle_token: Option<String>,
) -> Result<()> {
    let payer_key = program.payer();
    println!("payer_key: {}", payer_key.to_string());

    let (admin_pda, _bump) =
        Pubkey::find_program_address(&[ADMIN_STATE_SEED.as_bytes()], &program.id());
    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), name.as_bytes()],
        &program.id(),
    );
    let (collection_state, _bump) = Pubkey::find_program_address(
        &[COLLECTION_STATE_SEED.as_bytes(), collection_mint.as_ref()],
        &program.id(),
    );

    let mint_build = program
        .request()
        .accounts(fomo100_accounts::SetPrice {
            collection_state: collection_state,
            admin: payer_key,
            admin_state: admin_pda,
            collection_mint: collection_mint,
        })
        .args(fomo100_instructions::SetPrice {
            sol_price: sol_price,
            settle_token_price: settle_token_price,
            settle_token: settle_token.map(|x| Pubkey::from_str(&x).unwrap()),
        });

    println!(
        "mint_build: {:?}",
        mint_build.instructions()?.first().unwrap().to_owned()
    );
    let res = program
        .request()
        .instruction(mint_build.instructions()?.first().unwrap().to_owned())
        .send();

    println!("mint_res: {:?}", res);
    let mint_res = res?;
    println!("pda_state_address: {}", admin_pda.to_string());
    let collection_state = program.collection_state(&collection_mint)?;
    println!("collection_state: {:?}", collection_state);
    Ok(())
}

pub fn init_airdrop(
    program: &anchor_client::Program<Rc<Keypair>>,
    collection_name: String,
    init_amount: u32,
    init_sig: String,
    init_instruction_data: String,
) -> Result<()> {
    // get amount and sig from config;
    let payer_key = program.payer();
    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), collection_name.as_bytes()],
        &program.id(),
    );


    let (amount, sig, ed25519_instruction) = {
        let msg = format!("{}_{}_{}_{}", INIT_AIRDROP_SIGN_PREFIX,collection_mint.to_string(), payer_key.to_string(), init_amount);
        println!("start verify msg {}", msg);
        //此处固化了设置
        let sig_bytes: [u8; 64] = hex::decode(init_sig)?.try_into().unwrap();
        let ed25519_instruction = Instruction{
            program_id: solana_sdk::ed25519_program::id(),
            accounts: vec![],
            data: hex::decode(init_instruction_data).unwrap(),
        };
        (init_amount,sig_bytes, ed25519_instruction)
    };



    let (user_pda, _bump) = Pubkey::find_program_address(
        &[
            USER_STATE_SEED.as_bytes(),
            collection_mint.as_ref(),
            payer_key.as_ref(),
        ],
        &program.id(),
    );

    let (admin_state, _bump) =
        Pubkey::find_program_address(&[ADMIN_STATE_SEED.as_bytes()], &program.id());

    let init_build = program
        .request()
        .accounts(fomo100_accounts::InitAirdrop {
            payer: payer_key,
            user_state: user_pda,
            admin_state,
            collection_mint: collection_mint,
            rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
            ix_sysvar: Pubkey::from_str("Sysvar1nstructions1111111111111111111111111").unwrap(),
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100_instructions::InitAirdrop { amount, sig });

    let res = program
        .request()
        .instruction(ed25519_instruction)
        .instruction(init_build.instructions()?.first().unwrap().to_owned())
        .send();

    println!("mint_res: {:?}", res);
    let user_state = program.user_state(&collection_mint, &payer_key)?;
    println!("user_state: {:?}", user_state);
    Ok(())
}

pub fn get_admin(
    program: &anchor_client::Program<Rc<Keypair>>,
    elite_collection_name: String,
    core_collection_name: String,
) -> Result<()> {
    let admin_info = program.admin_state()?;
    println!("admin_info: {:?}", admin_info);
    let (elite_collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), elite_collection_name.as_bytes()],
        &program.id(),
    );
    let (core_collection_mint, _bump) = Pubkey::find_program_address(
            &[COLLECTION_MINT_SEED.as_bytes(), core_collection_name.as_bytes()],
        &program.id(),
    );
    let elite_collection_info = program.collection_state(&elite_collection_mint)?;
    let core_collection_info = program.collection_state(&core_collection_mint)?;
    println!("elite_collection_info {:?}", elite_collection_info);
    println!("core_collection_info {:?}", core_collection_info);
    Ok(())
}

pub fn init_airdrop_and_mint(
    program: &anchor_client::Program<Rc<Keypair>>,
    collection_name: String,
    pay_sol: bool,
    init_amount: u32,
    init_sig: String,
    init_instruction_data: String,
) -> Result<()> {
    println!("file: {}, line!{}",file!(),line!());
    let payer_key = program.payer();
    println!("file: {}, line!{}",file!(),line!());
    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), collection_name.as_bytes()],
        &program.id(),
    );
    println!("file: {}, line!{}",file!(),line!());

    let id = if let Ok(state) = program.user_state(&collection_mint, &payer_key) {
        init_amount + state.claimed_airdrop_count as u32 + 1
    } else {
        //防止和正常用户测试冲突，每有新的用户这里也要调整，比如A用户的你是100_000，B用户的你是200_000
        init_amount
    };
    let (nft_mint_key, _) = Pubkey::find_program_address(
        &[
            MINT_SEED.as_bytes(),
            collection_mint.as_ref(),
            id.to_le_bytes().as_ref(),
        ],
        &program.id(),
    );
    println!("nft mint key {}", nft_mint_key.to_string());

    println!("collection mint key {}", collection_mint.to_string());

    let (collection_authority, _) =
        Pubkey::find_program_address(&[COLLECTION_AUTHORITY_SEED.as_bytes()], &program.id());
    println!("collection authority {}", collection_authority.to_string());

    //use ata account for this nft mint
    let token_account = get_associated_token_address(&payer_key, &nft_mint_key);
    let metadata_address = find_metadata_pda(&nft_mint_key);
    let master_key = find_master_edition_pda(&nft_mint_key);

    println!(
        "metadata_address: {}\n,token_account: {}\n,nft_mint_key: {}\n,payer_key: {}\n,master_key: {}",
        metadata_address,
        token_account,
        nft_mint_key,
        payer_key,
        master_key
    );


    let collection_metadata = find_metadata_pda(&collection_mint);
    let collection_master_edition = find_master_edition_pda(&collection_mint);
    let (collection_state, _bump) = Pubkey::find_program_address(
        &[COLLECTION_STATE_SEED.as_bytes(), collection_mint.as_ref()],
        &program.id(),
    );

    let (user_state, _bump) = Pubkey::find_program_address(
        &[
            USER_STATE_SEED.as_bytes(),
            collection_mint.as_ref(),
            payer_key.as_ref(),
        ],
        &program.id(),
    );
    //print all accounts

    println!("collection mint key {}", collection_mint.to_string());
    println!("collection_metadata: {}", collection_metadata.to_string());
    println!(
        "collection_master_edition: {}",
        collection_master_edition.to_string()
    );
    println!("collection_authority: {}", collection_authority.to_string());
    let (admin_state, _bump) =
        Pubkey::find_program_address(&[ADMIN_STATE_SEED.as_bytes()], &program.id());

    let collection_info = program.collection_state(&collection_mint)?;
    let admin_info = program.admin_state()?;
    println!("admin_info: {:?}", admin_info);
    let user_settle_token_ata =
        get_associated_token_address(&payer_key, &collection_info.settle_token);
    let treasurer_settle_token_ata =
        get_associated_token_address(&admin_info.treasurer, &collection_info.settle_token);
    let accounts = fomo100_accounts::NftMintBySol{
        payer: payer_key,
        user_state,
        mint: nft_mint_key,
        token_account: token_account,
        associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID).unwrap(),
        rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
        system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
        token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
        metadata_program: Pubkey::from_str(MPL_TOKEN_METADATA_ACCOUNT).unwrap(),
        nft_metadata: metadata_address,
        nft_master_edition: master_key,
        collection_authority: collection_authority,
        collection_mint,
        collection_metadata,
        collection_master_edition,
        collection_state: collection_state,
        admin_state,
        treasurer: admin_info.treasurer,
    };
    println!("payer --> {:?}", accounts.payer);
    println!("user_state --> {:?}", accounts.user_state);
    println!("mint --> {:?}", accounts.mint);
    println!("token_account --> {:?}", accounts.token_account);
    println!("associated_token_program --> {:?}", accounts.associated_token_program);
    println!("rent --> {:?}", accounts.rent);
    println!("system_program --> {:?}", accounts.system_program);
    println!("token_program --> {:?}", accounts.token_program);
    println!("metadata_program --> {:?}", accounts.metadata_program);
    println!("nft_metadata --> {:?}", accounts.nft_metadata);
    println!("nft_master_edition --> {:?}", accounts.nft_master_edition);
    println!("collection_authority --> {:?}", accounts.collection_authority);
    println!("collection_mint --> {:?}", accounts.collection_mint);
    println!("collection_metadata --> {:?}", accounts.collection_metadata);
    println!("collection_master_edition --> {:?}", accounts.collection_master_edition);
    println!("collection_state --> {:?}", accounts.collection_state);
    println!("admin_state --> {:?}", accounts.admin_state);
    println!("treasurer --> {:?}", accounts.treasurer);
    
    
    
    
    let mint_build = if pay_sol {
        program
        .request()
        .accounts(fomo100_accounts::NftMintBySol{
            payer: payer_key,
            user_state,
            mint: nft_mint_key,
            token_account: token_account,
            associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID).unwrap(),
            rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            metadata_program: Pubkey::from_str(MPL_TOKEN_METADATA_ACCOUNT).unwrap(),
            nft_metadata: metadata_address,
            nft_master_edition: master_key,
            collection_authority: collection_authority,
            collection_mint,
            collection_metadata,
            collection_master_edition,
            collection_state: collection_state,
            admin_state,
            treasurer: admin_info.treasurer,
        })
        .args(fomo100_instructions::NftMintBySol{id})
    } else {
        program
        .request()
        .accounts(fomo100_accounts::NftMintByCoin{
            payer: payer_key,
            user_state,
            mint: nft_mint_key,
            token_account: token_account,
            associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID).unwrap(),
            rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
            system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            metadata_program: Pubkey::from_str(MPL_TOKEN_METADATA_ACCOUNT).unwrap(),
            nft_metadata: metadata_address,
            nft_master_edition: master_key,
            collection_authority: collection_authority,
            collection_mint,
            collection_metadata,
            collection_master_edition,
            collection_state: collection_state,
            admin_state,
            user_settle_token_ata,
            treasurer_settle_token_ata,
        })
        .args(fomo100_instructions::NftMintByCoin{})
    };

    let user_state_res = program.user_state(&collection_mint, &payer_key);
    //fixme: 报错等效于user_state不存在
    let mint_res = if user_state_res.is_err() {
        //todo: 提前生成签名，
        //判断是否有空投的签名
        let (amount, sig, ed25519_instruction) = {
            let msg = format!("{}_{}_{}_{}", INIT_AIRDROP_SIGN_PREFIX,collection_mint.to_string(), payer_key.to_string(), init_amount);
            println!("start verify msg {}", msg);
            //此处固化了设置
            let sig_bytes: [u8; 64] = hex::decode(init_sig)?.try_into().unwrap();
            let ed25519_instruction = Instruction{
                program_id: solana_sdk::ed25519_program::id(),
                accounts: vec![],
                data: hex::decode(init_instruction_data).unwrap(),
            };
            (init_amount, sig_bytes, ed25519_instruction)
        };
        let init_airdrop_build = program
            .request()
            .accounts(fomo100_accounts::InitAirdrop {
                payer: payer_key,
                user_state,
                admin_state,
                collection_mint: collection_mint,
                rent: Pubkey::from_str(SYSTEM_RENT_ID).unwrap(),
                ix_sysvar: Pubkey::from_str("Sysvar1nstructions1111111111111111111111111").unwrap(),
                system_program: Pubkey::from_str(SYSTEM_PROGRAM_ID).unwrap(),
            })
            .args(fomo100_instructions::InitAirdrop { amount, sig });
        program
            .request()
            .instruction(ed25519_instruction)
            .instruction(ComputeBudgetInstruction::set_compute_unit_limit(400_000))
            .instruction(
                init_airdrop_build
                    .instructions()?
                    .first()
                    .unwrap()
                    .to_owned(),
            )
            .instruction(mint_build.instructions()?.first().unwrap().to_owned())
            .send()
    } else {
        program
            .request()
            .instruction(ComputeBudgetInstruction::set_compute_unit_limit(400_000))
            .instruction(mint_build.instructions()?.first().unwrap().to_owned())
            .send()
    };

    println!("mint_res: {:?}", mint_res);
    let mint_res = mint_res?;
    println!("file: {},line {}", file!(), line!());

    println!("call res {}", mint_res);
    println!("nft mint key {}", nft_mint_key.to_string());
    let collection_info = program.collection_state(&collection_mint)?;
    println!("collection_info {:?}", collection_info);
    let user_state = program.user_state(&collection_mint, &payer_key)?;
    println!("user_state {:?}", user_state);
    Ok(())
}

use fomo100::constants::INIT_AIRDROP_SIGN_PREFIX;
pub fn sign_airdrop(
    program: &anchor_client::Program<Rc<Keypair>>,
    prikey: &String,
    collection_name: String,
    pubkey: String,
    amount: u32,
) -> Result<String> {
    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_MINT_SEED.as_bytes(), collection_name.as_bytes()],
        &program.id(),
    );
    let msg = format!("{}_{}_{}_{}", INIT_AIRDROP_SIGN_PREFIX,collection_mint.to_string(),pubkey, amount);
    //println!("start verify msg {}", msg);
    let validator = Keypair::from_base58_string(prikey);
    let validator_public_key = validator.pubkey().to_string();
    //println!("validator_public_key: {}", validator_public_key);
    let sig = validator.sign_message(&msg.as_bytes());
    //println!("sig={}", sig.to_string());
    let sig_bytes: [u8; 64] = sig.into();
    let sig_str = hex::encode(sig_bytes);
    let validator = ed25519_dalek::Keypair::from_bytes(&validator.to_bytes())?;
    let ed25519_instruction = solana_sdk::ed25519_instruction::new_ed25519_instruction(
        &validator,
        &msg.as_bytes(),
    );
    let instruction_data = hex::encode(ed25519_instruction.data);
    //println!("sig_str: {}, instruction_data: {}", sig_str, instruction_data);

    let json = json!({
        "collection_mint": collection_mint.to_string(),
        "pubkey": pubkey,
        "amount": amount,
        "sig": sig_str,
        "instruction_data": instruction_data
    });
    //println!("sig_bytes: {}", json);
    //签名包含本地签名和instruction_data
    Ok(format!("{}:{}", sig_str, instruction_data))
}


#[cfg(test)]
mod tests {
    // 导入测试模块
    use super::*;

    #[test]
    fn test_solpen_nft_mint_key() -> Result<()> {
        let prikey = "3DA3wS1AxjWJHUeeu329mMbBRZxAsoVw2DierNRrrF8bm2MDsKvLS6Qjs9LJVtC3EPx95CENhd2bS38LTf8Zt1cF";
        let collection_mint = Pubkey::from_str("rXoTKmKeFcgvTRE6vqfqw8Ebpo4Fdpg4DGwRYxozZf7").unwrap();
        let id = 10u32;
        let payer = Keypair::from_base58_string(&prikey);
        let cluster = Cluster::Custom("https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f".to_string(), "".to_string());
        let client = Client::new_with_options(cluster, Rc::new(payer), CommitmentConfig::confirmed());

        let program = client.program(Pubkey::from_str("4Th4Zf653GLACZ6yEEeharhv9JytsUYtPyXYAcA9freJ")?)?;

        let (nft_mint_key, _) = Pubkey::find_program_address(
            &[
                MINT_SEED.as_bytes(),
                collection_mint.as_ref(),
                id.to_le_bytes().as_ref(),
            ],
            &program.id(),
        );
      println!("nft_mint_key {}",nft_mint_key);
      let collection_master_edition = find_master_edition_pda(&collection_mint);
      println!("collection_master_edition {}",collection_master_edition);
      Ok(())
    }
}

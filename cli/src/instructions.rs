use anchor_client::anchor_lang::prelude::Pubkey;
use anchor_client::anchor_lang::Key;
use anchor_client::solana_sdk::signature::Keypair;
use anyhow::{anyhow, Result};
use fomo100::state::*;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use spl_associated_token_account::get_associated_token_address;
use std::rc::Rc;
use std::str::FromStr;
use std::u64;

use crate::state::State;
use crate::utils::get_lamport_balance;
use crate::{SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SPL_PROGRAM_ID, SYSTEM_PROGRAM_ID};

pub fn expand_pool_state<T: TryInto<Pubkey>>(
    program: &anchor_client::Program<Rc<Keypair>>,
    pool_state_pda: T,
) -> Result<()> {
    let payer_pubkey = program.payer();
    println!("payer_pubkey {}", payer_pubkey);
    let space = 10240;
    let lamports = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(space)?;
    let payer_balance = get_lamport_balance(&payer_pubkey)?;
    println!(
        "payer {} balance {}, need consume {} lamport",
        payer_pubkey, payer_balance, lamports
    );

    let pool_state_pda: Pubkey = pool_state_pda
        .try_into()
        .map_err(|e| anyhow!("pool_state_pda.try_into failed"))?;
    let init_res = program
        .request()
        .accounts(fomo100::accounts::ExpandPoolState {
            admin: payer_pubkey,
            pool_state: pool_state_pda,
        })
        .args(fomo100::instruction::ExpandPoolState {})
        .send()
        .unwrap();
    println!("expand_pool_state_sig {}", init_res.to_string());
    Ok(())
}

pub fn create_pool<T: TryInto<Pubkey>>(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: T,
    token_decimal: u8,
    min_stake_amount: u64,
    created_at: i64,
    round_period_secs: u32,
    round_reward: u64,
    unlock_period_secs: u64,
) -> Result<Pubkey> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);

    println!(
        "\npayer_pubkey={}\n,
        pool_state_pda={},
        pool_vault={},
        pool_store_pda={},
        dojo_mint_pubkey={},",
        payer_pubkey, pool_state_pda, pool_vault, pool_store_pda, dojo_mint_pubkey,
    );
    let init_res = program
        .request()
        .accounts(fomo100::accounts::CreatePool {
            admin: payer_pubkey,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID)
                .unwrap(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::CreatePool {
            token_decimal,
            min_stake_amount,
            created_at,
            round_period_secs,
            round_reward,
            unlock_period_secs,
        })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    let collection_state = program.pool_state(&dojo_mint_pubkey, created_at, round_period_secs)?;
    println!("collection_state: {:?}", collection_state);
    Ok(pool_state_pda)
}

pub fn set_admin(
    program: &anchor_client::Program<Rc<Keypair>>,
    new_admin: Option<String>,
    validator: Option<String>,
    treasurer: Option<String>,
) -> Result<()> {
    todo!()
}

pub fn set_round_reward<T: TryInto<Pubkey>>(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: T,
    created_at: i64,
    round_period_secs: u32,
    round_reward: u64,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let res = program
        .request()
        .accounts(fomo100::accounts::SetRoundReward {
            admin: payer_pubkey,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::SetRoundReward { round_reward })
        .send()
        .unwrap();
    println!("call res:  {}", res.to_string());
    let pool_state = program.pool_state(&dojo_mint_pubkey, created_at, round_period_secs)?;
    println!("pool_state: {:?}", pool_state);
    Ok(())
}

pub fn stake(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: &str,
    created_at: i64,
    round_period_secs: u32,
    amount: u64,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    //get pool pda
    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);
    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    // 2) get user pda
    let (user_state_pda, _bump) = Pubkey::find_program_address(
        &[
            payer_pubkey.key().as_ref(),
            pool_state_pda.key().as_ref(),
            USER_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let user_vault = get_associated_token_address(&user_state_pda, &dojo_mint_pubkey);

    let user_ata = get_associated_token_address(&payer_pubkey, &dojo_mint_pubkey);

    println!(
        "payer_pubkey={},
        pool_state_pda={},
        pool_vault={},
        dojo_mint_pubkey={},
        user_state_pda={},
        user_vault={},
        user_ata={}
        ",
        payer_pubkey,
        pool_state_pda,
        pool_vault,
        dojo_mint_pubkey,
        user_state_pda,
        user_vault,
        user_ata
    );
    let init_res = program
        .request()
        //max:
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(400_000))
        //max: 128KB
        .instruction(ComputeBudgetInstruction::request_heap_frame(64 * 1024))
        .accounts(fomo100::accounts::Stake {
            user: payer_pubkey,
            user_state: user_state_pda,
            user_vault,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            user_ata,
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            associated_token_program: Pubkey::from_str(SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID)
                .unwrap(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::Stake { amount })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    Ok(())
}

pub fn claim(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: &str,
    created_at: i64,
    round_period_secs: u32,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    //get pool pda
    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);
    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    // 2) get user pda
    let (user_state_pda, _bump) = Pubkey::find_program_address(
        &[
            payer_pubkey.key().as_ref(),
            pool_state_pda.key().as_ref(),
            USER_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let user_vault = get_associated_token_address(&user_state_pda, &dojo_mint_pubkey);

    let user_ata = get_associated_token_address(&payer_pubkey, &dojo_mint_pubkey);

    println!(
        "payer_pubkey={},
        pool_state_pda={},
        pool_vault={},
        dojo_mint_pubkey={},
        user_state_pda={},
        user_vault={},
        user_ata={}
        ",
        payer_pubkey,
        pool_state_pda,
        pool_vault,
        dojo_mint_pubkey,
        user_state_pda,
        user_vault,
        user_ata
    );
    let init_res = program
        .request()
        //max:
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(800_000))
        //max: 128KB
        .instruction(ComputeBudgetInstruction::request_heap_frame(64 * 1024))
        .accounts(fomo100::accounts::Claim {
            user: payer_pubkey,
            user_state: user_state_pda,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            user_ata,
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::Claim {
            created_at,
            round_period_secs,
        })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    Ok(())
}

pub fn unlock(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: &str,
    created_at: i64,
    round_period_secs: u32,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    //get pool pda
    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);
    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    // 2) get user pda
    let (user_state_pda, _bump) = Pubkey::find_program_address(
        &[
            payer_pubkey.key().as_ref(),
            pool_state_pda.key().as_ref(),
            USER_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let user_vault = get_associated_token_address(&user_state_pda, &dojo_mint_pubkey);

    let user_ata = get_associated_token_address(&payer_pubkey, &dojo_mint_pubkey);

    println!(
        "payer_pubkey={},
        pool_state_pda={},
        pool_vault={},
        dojo_mint_pubkey={},
        user_state_pda={},
        user_vault={},
        user_ata={}
        ",
        payer_pubkey,
        pool_state_pda,
        pool_vault,
        dojo_mint_pubkey,
        user_state_pda,
        user_vault,
        user_ata
    );
    let init_res = program
        .request()
        //max:
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(800_000))
        //max: 128KB
        .instruction(ComputeBudgetInstruction::request_heap_frame(64 * 1024))
        .accounts(fomo100::accounts::Unlock {
            user: payer_pubkey,
            user_state: user_state_pda,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            user_ata,
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::Unlock {
            created_at,
            round_period_secs,
        })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    Ok(())
}

pub fn cancel_unlock(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: &str,
    created_at: i64,
    round_period_secs: u32,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    //get pool pda
    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);
    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    // 2) get user pda
    let (user_state_pda, _bump) = Pubkey::find_program_address(
        &[
            payer_pubkey.key().as_ref(),
            pool_state_pda.key().as_ref(),
            USER_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let user_vault = get_associated_token_address(&user_state_pda, &dojo_mint_pubkey);

    let user_ata = get_associated_token_address(&payer_pubkey, &dojo_mint_pubkey);

    println!(
        "payer_pubkey={},
        pool_state_pda={},
        pool_vault={},
        dojo_mint_pubkey={},
        user_state_pda={},
        user_vault={},
        user_ata={}
        ",
        payer_pubkey,
        pool_state_pda,
        pool_vault,
        dojo_mint_pubkey,
        user_state_pda,
        user_vault,
        user_ata
    );
    let init_res = program
        .request()
        //max:
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(800_000))
        //max: 128KB
        .instruction(ComputeBudgetInstruction::request_heap_frame(64 * 1024))
        .accounts(fomo100::accounts::CancelUnlock {
            user: payer_pubkey,
            user_state: user_state_pda,
            pool_state: pool_state_pda.clone(),
            pool_store: pool_store_pda.clone(),
            user_ata,
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::CancelUnlock {
            created_at,
            round_period_secs,
        })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    Ok(())
}

pub fn unstake(
    program: &anchor_client::Program<Rc<Keypair>>,
    token_mint: &str,
    created_at: i64,
    round_period_secs: u32,
) -> Result<()> {
    let dojo_mint_pubkey: Pubkey = token_mint
        .try_into()
        .map_err(|e| anyhow!("token_mint.try_into failed"))?;
    let payer_pubkey = program.payer();

    //get pool pda
    let (pool_state_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    let pool_vault = get_associated_token_address(&pool_state_pda, &dojo_mint_pubkey);
    let (pool_store_pda, _bump) = Pubkey::find_program_address(
        &[
            dojo_mint_pubkey.key().as_ref(),
            created_at.to_be_bytes().as_ref(),
            round_period_secs.to_be_bytes().as_ref(),
            POOL_STORE_SEED.as_bytes(),
        ],
        &program.id(),
    );
    // 2) get user pda
    let (user_state_pda, _bump) = Pubkey::find_program_address(
        &[
            payer_pubkey.key().as_ref(),
            pool_state_pda.key().as_ref(),
            USER_STATE_SEED.as_bytes(),
        ],
        &program.id(),
    );

    let user_vault = get_associated_token_address(&user_state_pda, &dojo_mint_pubkey);

    let user_ata = get_associated_token_address(&payer_pubkey, &dojo_mint_pubkey);

    println!(
        "payer_pubkey={},
        pool_state_pda={},
        pool_vault={},
        dojo_mint_pubkey={},
        user_state_pda={},
        user_vault={},
        user_ata={}
        ",
        payer_pubkey,
        pool_state_pda,
        pool_vault,
        dojo_mint_pubkey,
        user_state_pda,
        user_vault,
        user_ata
    );
    let init_res = program
        .request()
        //max:
        .instruction(ComputeBudgetInstruction::set_compute_unit_limit(800_000))
        //max: 128KB
        .instruction(ComputeBudgetInstruction::request_heap_frame(64 * 1024))
        .accounts(fomo100::accounts::Unstake {
            user: payer_pubkey,
            user_state: user_state_pda,
            user_vault: user_vault.clone(),
            pool_state: pool_state_pda.clone(),
            user_ata,
            pool_vault: pool_vault,
            token_mint: dojo_mint_pubkey.clone(),
            token_program: Pubkey::from_str(SPL_PROGRAM_ID).unwrap(),
            system_program: Pubkey::from_str(&SYSTEM_PROGRAM_ID).unwrap(),
        })
        .args(fomo100::instruction::Unstake {
            created_at,
            round_period_secs,
        })
        .send()
        .unwrap();
    println!("init settings {}", init_res.to_string());
    Ok(())
}

pub fn get_admin(
    program: &anchor_client::Program<Rc<Keypair>>,
    elite_collection_name: String,
    core_collection_name: String,
) -> Result<()> {
    todo!()
}

pub fn init_airdrop_and_mint(
    program: &anchor_client::Program<Rc<Keypair>>,
    collection_name: String,
    pay_sol: bool,
    init_amount: u32,
    init_sig: String,
    init_instruction_data: String,
) -> Result<()> {
    todo!()
}

pub fn sign_airdrop(
    program: &anchor_client::Program<Rc<Keypair>>,
    prikey: &String,
    collection_name: String,
    pubkey: String,
    amount: u32,
) -> Result<String> {
    todo!()
}

// #[cfg(test)]
// mod tests {
//     // 导入测试模块
//     use super::*;

//     #[test]
//     fn test_solpen_nft_mint_key() -> Result<()> {
//         let prikey = "3DA3wS1AxjWJHUeeu329mMbBRZxAsoVw2DierNRrrF8bm2MDsKvLS6Qjs9LJVtC3EPx95CENhd2bS38LTf8Zt1cF";
//         let collection_mint =
//             Pubkey::from_str("rXoTKmKeFcgvTRE6vqfqw8Ebpo4Fdpg4DGwRYxozZf7").unwrap();
//         let id = 10u32;
//         let payer = Keypair::from_base58_string(&prikey);
//         let cluster = Cluster::Custom("https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f".to_string(), "".to_string());
//         let client =
//             Client::new_with_options(cluster, Rc::new(payer), CommitmentConfig::confirmed());

//         let program = client.program(Pubkey::from_str(
//             "4Th4Zf653GLACZ6yEEeharhv9JytsUYtPyXYAcA9freJ",
//         )?)?;

//         let (nft_mint_key, _) = Pubkey::find_program_address(
//             &[
//                 MINT_SEED.as_bytes(),
//                 collection_mint.as_ref(),
//                 id.to_le_bytes().as_ref(),
//             ],
//             &program.id(),
//         );
//         println!("nft_mint_key {}", nft_mint_key);
//         let collection_master_edition = find_master_edition_pda(&collection_mint);
//         println!("collection_master_edition {}", collection_master_edition);
//         Ok(())
//     }
// }

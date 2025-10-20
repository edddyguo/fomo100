use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_client::client_error::reqwest;
use tokio::runtime::Runtime;

use crate::utils::is_test;

pub const KEY_ID: &str = "c899e4cab450d5f7ce97e07b";
pub const SOLPEN_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJkYXRhIjoiRHlrc0dVWjd3cEVoaTZyYUFSUThTRVhDelV1eGJpQld5NDJoRVVyS1RRQmYiLCJpYXQiOjE3NDIwNDEzNTgsImV4cCI6MzkxOTMyMTM1OH0.vDyRAbQjOyP3Cl-xYF30h9KNdFCjZ-707LdiLcciBUo";
pub const NFT_CLAIM_URL: &str = "https://api.solpen.io/api/accounts/nft_claim_info";
pub const NFT_CLAIM_URL_TEST: &str = "https://testapi.solpen.shop/api/accounts/nft_claim_info";
pub const SET_NFT_CLAIM_SIG_URL: &str = "https://api.solpen.io/api/accounts/set_nft_claim_sig";
pub const SET_NFT_CLAIM_SIG_URL_TEST: &str = "https://testapi.solpen.shop/api/accounts/set_nft_claim_sig";

#[derive(Deserialize,Debug,Clone)]
pub struct NftClaimInfo {
    pub address: String,
    //solpen core
    pub nft_amount: u32,
    pub nft_sig: String,
    //solpen elite
    pub nft_premium_amount: u32,
    pub nft_premium_sig: String,
    pub premium_nft_ids: Vec<u32>,
    pub nft_ids: Vec<u32>,
}


#[derive(Deserialize,Debug,Clone)]
pub struct GetClaimedListResponse {
    pub message: String,
    pub data: Vec<NftClaimInfo>,
    pub nft_ts: u64,
    pub premium_nft_ts: u64,
}

#[derive(Deserialize,Debug)]
struct Body<T> {
    code: u32,
    msg: String,
    pub data: Option<T>,
}

pub fn get_claimed_list() -> Vec<NftClaimInfo>{
    let url = if is_test() {
        NFT_CLAIM_URL_TEST
    }else{
        NFT_CLAIM_URL
    };
    let rt = Runtime::new().unwrap();
    // 使用 block_on 执行异步任务并获取结果
    let result = rt.block_on(async {
        let client = reqwest::Client::new();        
        let res = client.get(url)
        .header("Authorization", format!("bearer {}", SOLPEN_TOKEN))
        .send().await.unwrap();

        let body_str = res.text().await.unwrap();
        //println!("body_str {},body_str_size {}",body_str,body_str.len());
        let body_res = serde_json::from_str::<Body<GetClaimedListResponse>>(&body_str).unwrap();
        //println!("body_res {:?}",body_res);
        body_res.data.unwrap().data
    });
    result
}


#[derive(Deserialize,Debug,Serialize,Clone)]
pub struct SetNftClaimSigRequest {
    pub address: String,
    pub nft_sig: String,
    pub nft_premium_sig: String
}

pub fn set_claimed_sig(proofs: Vec<SetNftClaimSigRequest>){
    let url = if is_test() {
        SET_NFT_CLAIM_SIG_URL_TEST
    }else{
        SET_NFT_CLAIM_SIG_URL
    };
    let client = reqwest::Client::new();
    let body = json!({
        "key_id": KEY_ID,
        "data": proofs
    }).to_string();
    println!("url {} request_body {}",url,body);

    let rt = Runtime::new().unwrap();

    // 使用 block_on 执行异步任务并获取结果
    rt.block_on(async {
        let res = client.post(url)
            .header("Authorization", format!("bearer {}", SOLPEN_TOKEN))
            .header("Content-Type", "application/json")
            .body(body)
            .send().await.unwrap();
    
        let body_str = res.text().await.unwrap();
        println!("body_str {}",body_str);
        let body_res = serde_json::from_str::<Body<Value>>(&body_str).unwrap();
        if body_res.code == 200 {
            println!("set_claimed_sig success");
            ()
        }else{
            panic!("set_claimed_sig failed");
        }
    });
    
}
#[cfg(test)]
mod tests {
    // 导入测试模块
    use super::*;

    #[test]
    fn test_solpen_nft_brace() {
      let claimed_list = get_claimed_list();
      println!("claimed_list {:#?}",claimed_list);
      //7RQoVoLFhohAM95fcQHzycf5XoNaXKSggXSJUPmfkCm6
      let proofs = SetNftClaimSigRequest {
        address: "3ArFqke4siVqnfDMo2cUqfqh5Gn23EMceCkUddUQUSA8".to_string(),
        nft_sig: "111".to_string(),
        nft_premium_sig: "222".to_string(),
      };
      set_claimed_sig(vec![proofs]);
      let claimed_list = get_claimed_list();
      println!("claimed_list {:#?}",claimed_list);
    }
}

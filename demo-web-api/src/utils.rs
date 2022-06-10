use crate::error::AppError;
use anchor_lang::{
    prelude::Pubkey,
    InstructionData, ToAccountMetas,
    {
        solana_program::{instruction::Instruction, sysvar},
        system_program,
    },
};
use bpl_token_metadata::{
    accounts::MintPromoToken as mint_promo_token_accounts,
    instruction::MintPromoToken as mint_promo_token_instruction,
    utils::{
        find_admin_address, find_associated_token_address, find_authority_address,
        find_promo_address,
    },
};
use futures::join;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_sdk::hash::Hash;
use std::str::FromStr;

pub async fn create_transfer_promo_instruction(
    wallet: Pubkey,
    mint: Pubkey,
    promo_owner: Pubkey,
) -> Result<Instruction, AppError> {
    let (
        (authority, _auth_bump),
        (promo, _promo_bump),
        (admin_settings, _admin_bump),
        token_account,
    ) = join!(
        find_authority_address(),
        find_promo_address(&mint),
        find_admin_address(),
        find_associated_token_address(&wallet, &mint)
    );

    let accounts = mint_promo_token_accounts {
        payer: wallet,
        mint,
        promo_owner,
        authority,
        promo,
        admin_settings,
        token_account,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = mint_promo_token_instruction {}.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub struct Solana {
    pub base_url: url::Url,
    pub client: reqwest::Client,
}

impl Solana {
    pub async fn get_latest_blockhash(&self) -> Result<Hash, AppError> {
        let mut config = serde_json::Map::new();
        config.insert(
            "commitment".to_string(),
            Value::String("confirmed".to_string()),
        );

        let post_object = PostObject {
            method: String::from("getLatestBlockhash"),
            ..Default::default()
        };

        let result: Value = self
            .client
            .post(self.base_url.clone())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await?;

        let hash_str = result["result"]["value"]["blockhash"].as_str().unwrap();
        let hash = Hash::from_str(hash_str)?;
        Ok(hash)
    }

    pub async fn post_transaction(&self, tx_str: &str) -> Result<SendTransResultObject, AppError> {
        let post_object = PostObject {
            params: vec![Value::String(tx_str.to_string())],
            ..Default::default()
        };

        let result: SendTransResultObject = self
            .client
            .post(self.base_url.clone())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::SolanaPostError(e))?;

        tracing::debug!("post_transaction_result {:?}", &result);

        Ok(result)
    }

    pub async fn post_transaction_test(&self, tx_str: &str) -> Result<Value, AppError> {
        let post_object = PostObject {
            params: vec![Value::String(tx_str.to_string())],
            ..Default::default()
        };

        // let result: SendTransResultObject = self

        let result: Value = self
            .client
            .post(self.base_url.clone())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::SolanaPostError(e))?;

        tracing::debug!("post_transaction_result {:?}", &result);

        Ok(result)
    }

    pub async fn get_transaction(
        &self,
        sig_string: &str,
    ) -> Result<GetTransResultObject, AppError> {
        let mut config = serde_json::Map::new();
        config.insert("encoding".to_string(), Value::String("json".to_string()));
        config.insert(
            "commitment".to_string(),
            Value::String("confirmed".to_string()),
        );

        let post_object = PostObject {
            method: String::from("getTransaction"),
            params: vec![Value::String(sig_string.to_string()), Value::Object(config)],
            ..Default::default()
        };

        let result: GetTransResultObject = self
            .client
            .post(self.base_url.clone())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await
            .map_err(|e| AppError::SolanaGetError(e))?;
        Ok(result)
    }

    pub async fn request_airdrop(&self, pubkey: String, lamports: u64) -> Result<String, AppError> {
        let post_object = PostObject {
            method: "requestAirdrop".to_string(),
            params: vec![json!(pubkey), json!(lamports)],
            ..Default::default()
        };

        let result: Value = self
            .client
            .post(self.base_url.clone())
            .json(&post_object)
            .send()
            .await?
            .json()
            .await?;

        println!("{}", &result);
        Ok(result["result"].as_str().unwrap().to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostObject {
    pub jsonrpc: String,
    pub id: usize,
    pub method: String,
    pub params: Vec<Value>,
}

impl Default for PostObject {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendTransaction".to_string(),
            params: Vec::<Value>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTransResultObject {
    pub jsonrpc: String,
    pub result: String,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetTransResultObject {
    pub jsonrpc: String,
    pub result: Option<GetTransResultResult>,
    pub block_time: Option<u64>,
    pub id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Status {
    pub Ok: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransResultResult {
    pub meta: Meta,
    pub slot: u64,
    pub transaction: Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub err: Option<u8>,
    pub fee: u64,
    pub inner_instructions: Vec<u8>,
    pub post_balances: Vec<u64>,
    pub post_token_balances: Vec<u64>,
    pub pre_balances: Vec<u64>,
    pub pre_token_balances: Vec<u64>,
    pub status: Status,
}

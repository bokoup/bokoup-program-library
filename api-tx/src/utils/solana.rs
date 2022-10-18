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
    accounts::{
        BurnDelegatedPromoToken as burn_delegated_promo_token_accounts,
        CreatePromo as create_promo_accounts, DelegatePromoToken as delegate_promo_token_accounts,
        MintPromoToken as mint_promo_token_accounts,
    },
    instruction::{
        BurnDelegatedPromoToken as burn_delegated_promo_token_instruction,
        CreatePromo as create_promo_instruction,
        DelegatePromoToken as delegate_promo_token_instruction,
        MintPromoToken as mint_promo_token_instruction,
    },
    state::{DataV2, Promo},
    utils::{
        find_admin_address, find_associated_token_address, find_authority_address,
        find_metadata_address, find_promo_address,
    },
};
use serde::{Deserialize, Serialize};

use serde_json::{json, Value};
use solana_sdk::{commitment_config::CommitmentLevel, hash::Hash};
use std::str::FromStr;

pub fn create_create_promo_instruction(
    payer: Pubkey,
    mint: Pubkey,
    platform: Pubkey,
    name: String,
    symbol: String,
    uri: String,
    max_mint: Option<u32>,
    max_burn: Option<u32>,
    is_mutable: bool,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (authority, _auth_bump) = find_authority_address();
    let (promo, _promo_bump) = find_promo_address(&mint);
    let (metadata, _metadata_bump) = find_metadata_address(&mint);
    let (admin_settings, _admin_bump) = find_admin_address();

    let accounts = create_promo_accounts {
        payer,
        mint,
        metadata,
        authority,
        promo,
        platform,
        admin_settings,
        metadata_program: mpl_token_metadata::ID,
        token_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let promo_data = Promo {
        owner: payer,
        mint,
        metadata,
        mint_count: 0,
        burn_count: 0,
        max_mint,
        max_burn,
    };

    let metadata_data = DataV2 {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let data = create_promo_instruction {
        promo_data,
        metadata_data,
        is_mutable,
        memo,
    }
    .data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_mint_promo_instruction(
    payer: Pubkey,
    token_owner: Pubkey,
    mint: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (authority, _auth_bump) = find_authority_address();
    let (promo, _promo_bump) = find_promo_address(&mint);
    let (admin_settings, _admin_bump) = find_admin_address();
    let token_account = find_associated_token_address(&token_owner, &mint);

    let accounts = mint_promo_token_accounts {
        payer,
        token_owner,
        mint,
        authority,
        promo,
        admin_settings,
        token_account,
        token_program: anchor_spl::token::ID,
        memo_program: spl_memo::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = mint_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_delegate_promo_instruction(
    payer: Pubkey,
    token_owner: Pubkey,
    mint: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (authority, _auth_bump) = find_authority_address();
    let (promo, _promo_bump) = find_promo_address(&mint);
    let token_account = find_associated_token_address(&token_owner, &mint);

    let accounts = delegate_promo_token_accounts {
        payer,
        token_owner,
        authority,
        promo,
        token_account,
        memo_program: spl_memo::ID,
        token_program: anchor_spl::token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = delegate_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

pub fn create_burn_delegated_promo_instruction(
    payer: Pubkey,
    token_owner: Pubkey,
    mint: Pubkey,
    platform: Pubkey,
    memo: Option<String>,
) -> Result<Instruction, AppError> {
    let (authority, _auth_bump) = find_authority_address();
    let (promo, _promo_bump) = find_promo_address(&mint);
    let (admin_settings, _admin_bump) = find_admin_address();
    let token_account = find_associated_token_address(&token_owner, &mint);

    let accounts = burn_delegated_promo_token_accounts {
        payer,
        mint,
        authority,
        promo,
        platform,
        admin_settings,
        token_account,
        memo_program: spl_memo::ID,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = burn_delegated_promo_token_instruction { memo }.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

// Needed to do this since nonblocking client not avaiable in 1.9.20.
pub struct Solana {
    pub cluster: Cluster,
    pub commitment: CommitmentLevel,
    pub client: reqwest::Client,
}

impl Solana {
    pub async fn get_latest_blockhash(&self) -> Result<Hash, AppError> {
        let mut config = serde_json::Map::new();
        config.insert(
            "commitment".to_string(),
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            method: String::from("getLatestBlockhash"),
            ..Default::default()
        };

        let result: Value = self
            .client
            .post(self.cluster.url())
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
            params: vec![
                Value::String(tx_str.to_string()),
                json!({"encoding": "base64"}),
            ],
            ..Default::default()
        };

        let result: SendTransResultObject = self
            .client
            .post(self.cluster.url())
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
            params: vec![
                Value::String(tx_str.to_string()),
                json!({"encoding": "base64"}),
            ],
            ..Default::default()
        };

        // let result: SendTransResultObject = self

        let result: Value = self
            .client
            .post(self.cluster.url())
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
            Value::String(self.commitment.to_string()),
        );

        let post_object = PostObject {
            method: String::from("getTransaction"),
            params: vec![Value::String(sig_string.to_string()), Value::Object(config)],
            ..Default::default()
        };

        let result: GetTransResultObject = self
            .client
            .post(self.cluster.url())
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
            .post(self.cluster.url())
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

// Copied here to avoid depending on anchor client and in turn solana client which was bonking
// the cloud run image because of the usb interface hidapi.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cluster {
    Testnet,
    Mainnet,
    Devnet,
    Localnet,
    Debug,
    Custom(String, String),
}

impl Default for Cluster {
    fn default() -> Self {
        Cluster::Localnet
    }
}

impl FromStr for Cluster {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Cluster, AppError> {
        match s.to_lowercase().as_str() {
            "t" | "testnet" => Ok(Cluster::Testnet),
            "m" | "mainnet" => Ok(Cluster::Mainnet),
            "d" | "devnet" => Ok(Cluster::Devnet),
            "l" | "localnet" => Ok(Cluster::Localnet),
            "g" | "debug" => Ok(Cluster::Debug),
            _ if s.starts_with("http") => {
                let http_url = s;

                let mut ws_url = url::Url::parse(http_url).map_err(AppError::UrlParseError) ?;
                if let Some(port) = ws_url.port() {
                    ws_url.set_port(Some(port + 1))
                        .map_err(|_| AppError::GenericError("Unable to set port".to_string()))?;
                }
                if ws_url.scheme() == "https" {
                    ws_url.set_scheme("wss")
                        .map_err(|_| AppError::GenericError("Unable to set scheme".to_string()))?;
                } else {
                    ws_url.set_scheme("ws")
                        .map_err(|_| AppError::GenericError("Unable to set scheme".to_string()))?;
                }


                Ok(Cluster::Custom(http_url.to_string(), ws_url.to_string()))
            }
            _ => Err(AppError::GenericError(
                "Cluster must be one of [localnet, testnet, mainnet, devnet] or be an http or https url\n".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let clust_str = match self {
            Cluster::Testnet => "testnet",
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Localnet => "localnet",
            Cluster::Debug => "debug",
            Cluster::Custom(url, _ws_url) => url,
        };
        write!(f, "{}", clust_str)
    }
}

impl Cluster {
    pub fn url(&self) -> &str {
        match self {
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Testnet => "https://api.testnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Debug => "http://34.90.18.145:8899",
            Cluster::Custom(url, _ws_url) => url,
        }
    }
    pub fn ws_url(&self) -> &str {
        match self {
            Cluster::Devnet => "wss://api.devnet.solana.com",
            Cluster::Testnet => "wss://api.testnet.solana.com",
            Cluster::Mainnet => "wss://api.mainnet-beta.solana.com",
            Cluster::Localnet => "ws://127.0.0.1:9000",
            Cluster::Debug => "ws://34.90.18.145:9000",
            Cluster::Custom(_url, ws_url) => ws_url,
        }
    }
}

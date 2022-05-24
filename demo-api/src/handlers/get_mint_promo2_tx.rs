use anchor_lang::prelude::Pubkey;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::{collections::HashMap, str::FromStr};

use crate::{error::AppError, utils::create_transfer_promo_instruction};

pub async fn handler(
    Json(data): Json<Data>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ResponseData>, AppError> {
    let wallet = Pubkey::from_str(&data.account)?;
    log::debug!("get_mint_promo2_tx: {:?}", params);
    let mint = Pubkey::from_str("9cppW5ugbEHygEicY8vWcgyCRNqkbdTiwjqtBDpH7913")?;
    let instruction = create_transfer_promo_instruction(wallet, mint).await?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&wallet));
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message: "Approve to receive Promo 2.".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    pub transaction: String,
    pub message: String,
}

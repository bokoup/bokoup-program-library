use anchor_lang::prelude::Pubkey;
use axum::Json;
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

use crate::{error::AppError, utils::create_transfer_promo_instruction};

pub async fn handler(Json(data): Json<Data>) -> Result<Json<ResponseData>, AppError> {
    let wallet = Pubkey::from_str(&data.account)?;
    let mint = Pubkey::from_str(&data.mint)?;
    let instruction = create_transfer_promo_instruction(wallet, mint).await?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&wallet));
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message: "You've got a promo!".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
    pub mint: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    pub transaction: String,
    pub message: String,
}

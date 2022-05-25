use anchor_lang::prelude::Pubkey;
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

use crate::{error::AppError, handlers::Params, utils::create_transfer_promo_instruction};

pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        promo_name,
    }): Path<Params>,
) -> Result<Json<ResponseData>, AppError> {
    let wallet = Pubkey::from_str(&data.account)?;
    log::debug!(
        "get_mint_promo:mint_string: {}, promo_name: {}",
        mint_string,
        promo_name
    );
    let mint = Pubkey::from_str(&mint_string)?;
    let instruction = create_transfer_promo_instruction(wallet, mint).await?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&wallet));
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message: format!("Approve to receive {}.", promo_name),
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

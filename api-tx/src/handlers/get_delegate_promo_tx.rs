use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError, handlers::Params, utils::solana::create_delegate_promo_instruction, State,
};

pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        message,
        memo,
    }): Path<Params>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ResponseData>, AppError> {
    tracing::debug!(mint_string = mint_string, message, memo);

    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.promo_owner.pubkey();
    let mint = Pubkey::from_str(&mint_string)?;
    let instruction = create_delegate_promo_instruction(payer, token_owner, mint, memo)?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.promo_owner], latest_blockhash)?;

    // let notification_data = NotificationData {
    //     data: base64::encode(tx.signatures[0]),
    //     ..Default::default()
    // };

    // state
    //     .clover
    //     .post_device_notification(merchant_id.unwrap().as_str(), notification_data)
    //     .await?;

    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message,
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

// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications
// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications

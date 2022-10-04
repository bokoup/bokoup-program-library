use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError,
    handlers::Params,
    utils::{clover::NotificationData, solana::create_burn_promo_instruction},
    State,
};

pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        promo_name,
        merchant_id,
    }): Path<Params>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ResponseData>, AppError> {
    let wallet = Pubkey::from_str(&data.account)?;
    tracing::debug!(
        "get_burn_promo:mint_string: {}, promo_name: {}, merchant_id: {}",
        mint_string,
        promo_name,
        merchant_id.clone().unwrap_or("none".to_string())
    );
    let mint = Pubkey::from_str(&mint_string)?;
    let instruction =
        create_burn_promo_instruction(wallet, mint, state.promo_owner.pubkey(), state.platform)
            .await?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&wallet));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.promo_owner], latest_blockhash)?;

    let notification_data = NotificationData {
        data: base64::encode(tx.signatures[1]),
        ..Default::default()
    };

    state
        .clover
        .post_device_notification(merchant_id.unwrap().as_str(), notification_data)
        .await?;
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseData {
        transaction,
        message: format!("Approve to use {}.", promo_name),
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

use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError, handlers::Params, utils::solana::create_burn_delegated_promo_instruction,
    State,
};

pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        message,
        memo,
    }): Path<Params>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Value>, AppError> {
    tracing::debug!(mint_string = mint_string, message, memo);

    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.promo_owner.pubkey();
    let group_seed = Pubkey::new_unique();

    let mint = Pubkey::from_str(&mint_string)?;
    let instruction = create_burn_delegated_promo_instruction(
        payer,
        group_seed,
        token_owner,
        mint,
        state.platform,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.sign(&[&state.promo_owner], latest_blockhash);

    let serialized = bincode::serialize(&tx)?;
    let tx_str = base64::encode(serialized);
    let result = state.solana.post_transaction_test(&tx_str).await?;

    tracing::debug!(result = format!("{:?}", result));

    Ok(Json(result))
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

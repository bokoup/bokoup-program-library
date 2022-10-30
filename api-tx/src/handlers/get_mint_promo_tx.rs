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
    utils::data::get_group,
    utils::{data::PROMO_GROUP_QUERY, solana::create_mint_promo_instruction},
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
) -> Result<Json<ResponseData>, AppError> {
    tracing::debug!(mint_string = mint_string, message = message, memo = memo);

    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.promo_owner.pubkey();
    let mint = Pubkey::from_str(&mint_string)?;

    let query =
        serde_json::json!({ "query": PROMO_GROUP_QUERY, "variables": {"mint": mint_string}});
    let result: serde_json::Value = state
        .solana
        .client
        .post(&state.data_url)
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    let group = get_group(&payer, &result)?;

    let instruction = create_mint_promo_instruction(payer, group, token_owner, mint, memo)?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.promo_owner], latest_blockhash)?;
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseData {
    pub transaction: String,
    pub message: String,
}

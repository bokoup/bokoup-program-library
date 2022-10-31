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
    utils::data::get_group_from_promo_group_query,
    utils::{data::MINT_QUERY, solana::create_mint_promo_instruction},
    State,
};

use super::PayResponse;

/// Handles merchant having added platform signer to group members to pay for transactions
/// so users can mint directly without merchant approval. `token_owner` address assumed
/// to be in the body of the request.
pub async fn handler(
    Json(data): Json<Data>,
    Path(Params {
        mint_string,
        message,
        memo,
    }): Path<Params>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(mint_string = mint_string, message = message, memo = memo);

    let token_owner = Pubkey::from_str(&data.account)?;
    let payer = state.platform_signer.pubkey();
    let mint = Pubkey::from_str(&mint_string)?;

    let query = serde_json::json!({ "query": MINT_QUERY, "variables": {"mint": mint_string}});
    let result: serde_json::Value = state
        .solana
        .client
        .post(&state.data_url)
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    let group = match get_group_from_promo_group_query(&payer, &result) {
        Ok(group) => Ok(group),
        Err(e) => {
            tracing::error!(error = e.to_string());
            Err(e)
        }
    }?;

    let instruction = create_mint_promo_instruction(payer, group, token_owner, mint, memo)?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.try_partial_sign(&[&state.platform_signer], latest_blockhash)?;
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message,
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
}

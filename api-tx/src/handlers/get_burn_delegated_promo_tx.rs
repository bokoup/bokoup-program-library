use anchor_lang::prelude::Pubkey;
use axum::{
    extract::{Extension, Path},
    Json,
};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::{str::FromStr, sync::Arc};

use crate::{
    error::AppError,
    utils::{
        data::{get_mint_owner_group_from_token_account_query, TOKEN_ACCOUNT_QUERY},
        solana::create_burn_delegated_promo_instruction,
    },
    State,
};

use super::{BurnDelegatedParams, PayResponse};

pub async fn handler(
    Json(data): Json<Data>,
    Path(BurnDelegatedParams {
        token_account_string,
        message,
        memo,
    }): Path<BurnDelegatedParams>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(token_account_string, message, memo);

    let payer = Pubkey::from_str(&data.account)?;

    let query = serde_json::json!({ "query": TOKEN_ACCOUNT_QUERY, "variables": {"id": token_account_string}});
    let result: serde_json::Value = state
        .solana
        .client
        .post(&state.data_url.to_string())
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    let (mint, token_owner, group) =
        get_mint_owner_group_from_token_account_query(&payer, &result)?;

    let instruction = create_burn_delegated_promo_instruction(
        payer,
        group,
        token_owner,
        mint,
        state.platform,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let recent_blockhash = state.solana.get_latest_blockhash().await?;
    tx.message.recent_blockhash = recent_blockhash;

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

// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications
// https://sandbox.dev.clover.com/v3/apps/MAC8DQKWCCB1R/merchants/XKDCJNW9JXGM1/notifications

use anchor_lang::prelude::Pubkey;
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

use crate::{error::AppError, utils::solana::create_create_promo_group_instruction};

use super::{PayResponse, PromoGroupParams};

pub async fn handler(
    Json(data): Json<Data>,
    Path(PromoGroupParams {
        group_seed,
        members,
        lamports,
        memo,
    }): Path<PromoGroupParams>,
) -> Result<Json<PayResponse>, AppError> {
    tracing::debug!(group_seed, lamports);

    let payer = Pubkey::from_str(&data.account)?;
    let group_seed = Pubkey::from_str(&group_seed)?;
    let members: Vec<Pubkey> = members
        .iter()
        .map(|s| Pubkey::from_str(s))
        .collect::<Result<Vec<Pubkey>, _>>()?;

    let instruction =
        create_create_promo_group_instruction(payer, group_seed, members, lamports, memo)?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&payer));
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(PayResponse {
        transaction,
        message: "Create promoGroup".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Data {
    pub account: String,
}

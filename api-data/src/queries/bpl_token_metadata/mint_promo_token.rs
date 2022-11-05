use borsh::de::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("mint_promo_token_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();
    let memo = if let Ok(args) =
        bpl_token_metadata::instruction::MintPromoToken::try_from_slice(&data[8..])
    {
        args.memo.map(|m| {
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(&m) {
                result
            } else {
                serde_json::json!({ "memo": m })
            }
        })
    } else {
        None
    };

    let signature = signature.to_string();
    let payer = &accounts[0];
    let promo_group = &accounts[1];
    let token_owner = &accounts[2];
    let mint = &accounts[3];
    let authority = &accounts[4];
    let promo = &accounts[5];
    let token_account = &accounts[6];
    let slot = slot as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &signature,
                payer,
                promo_group,
                token_owner,
                mint,
                authority,
                promo,
                token_account,
                &Json::<Option<serde_json::Value>>(memo),
                &slot,
            ],
        )
        .await;
    match result {
        Ok(row) => {
            let insert = row.get::<usize, Option<bool>>(0).unwrap();
            info!(signature = signature.as_str(), insert);
        }
        Err(error) => {
            error!(signature = signature.as_str(), ?error);
        }
    }
}

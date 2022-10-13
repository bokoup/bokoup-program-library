use crate::Memo;
use borsh::de::BorshDeserialize;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("create_promo_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();
    let memo: Option<Memo> =
        if let Ok(args) = bpl_token_metadata::instruction::CreatePromo::try_from_slice(data) {
            args.memo.map(Into::into)
        } else {
            None
        };

    let signature = signature.to_string();
    let payer = &accounts[0];
    let mint = &accounts[1];
    let metadata = &accounts[2];
    let authority = &accounts[3];
    let promo = &accounts[4];
    let platform = &accounts[5];
    let admin_settings = &accounts[6];
    let slot = slot as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &signature,
                payer,
                mint,
                metadata,
                authority,
                promo,
                platform,
                admin_settings,
                &Json::<Option<Memo>>(memo),
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
            error!(signature = signature.as_str(), ?error,);
        }
    }
}

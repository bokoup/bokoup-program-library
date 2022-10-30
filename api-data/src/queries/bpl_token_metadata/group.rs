use bpl_token_metadata::state::PromoGroup;
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("group_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &PromoGroup,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let owner = account.owner.to_string();
    let seed = account.seed.to_string();
    let nonce = account.nonce as i32;
    let members = account.members.iter().map(ToString::to_string).collect();
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &owner,
                &seed,
                &nonce,
                &Json::<Vec<String>>(members),
                &slot,
                &write_version,
            ],
        )
        .await;
    match result {
        Ok(row) => {
            let insert = row.get::<usize, Option<bool>>(0).unwrap();
            info!(id = id.as_str(), insert);
        }
        Err(error) => {
            error!(id = id.as_str(), ?error);
        }
    }
}

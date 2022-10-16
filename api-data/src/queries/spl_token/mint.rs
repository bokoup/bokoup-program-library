use spl_token::state::Mint;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("mint_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(client: &Client, key: &[u8], account: &Mint, slot: u64, write_version: u64) {
    let id = bs58::encode(key).into_string();
    let freeze_authority: Option<String> = account.freeze_authority.map(|p| p.to_string()).into();
    let mint_authority: Option<String> = account.mint_authority.map(|p| p.to_string()).into();
    let supply = account.supply as i64;
    let decimals = account.decimals as i32;
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &freeze_authority,
                &mint_authority,
                &account.is_initialized,
                &supply,
                &decimals,
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
        Err(ref error) => {
            error!(id = id.as_str(), ?error);
        }
    }
}

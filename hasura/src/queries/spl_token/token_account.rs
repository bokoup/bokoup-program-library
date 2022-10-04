use spl_token::state::{Account, AccountState};
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("token_account_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(client: &Client, key: &[u8], account: &Account, slot: u64, write_version: u64) {
    let id = bs58::encode(key).into_string();
    let mint = account.mint.to_string();
    let owner = account.owner.to_string();
    let amount = account.amount as i64;
    let delegate: Option<String> = account.delegate.map(|p| p.to_string()).into();
    let state = match account.state {
        AccountState::Uninitialized => "Uninitialized",
        AccountState::Initialized => "Initialized",
        AccountState::Frozen => "Frozen",
    };

    let is_native: Option<i64> = account.is_native.map(|k| k as i64).into();
    let delegated_amount = account.delegated_amount as i64;
    let close_authority: Option<String> = account.close_authority.map(|p| p.to_string()).into();
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &mint,
                &owner,
                &amount,
                &delegate,
                &state,
                &is_native,
                &delegated_amount,
                &close_authority,
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

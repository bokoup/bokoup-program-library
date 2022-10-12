use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("burn_delegated_promo_token_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    signature: &Signature,
    accounts: &Vec<Pubkey>,
    _data: &[u8],
    slot: u64,
) {
    let accounts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let signature = signature.to_string();
    let payer = &accounts[0];
    let mint = &accounts[1];
    let authority = &accounts[2];
    let promo = &accounts[3];
    let platform = &accounts[4];
    let admin_settings = &accounts[5];
    let token_account = &accounts[6];
    let slot = slot as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &signature,
                payer,
                mint,
                authority,
                promo,
                platform,
                admin_settings,
                token_account,
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

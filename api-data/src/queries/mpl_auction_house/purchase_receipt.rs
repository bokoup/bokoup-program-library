use mpl_auction_house::receipt::PurchaseReceipt;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("purchase_receipt_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &PurchaseReceipt,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let bookkeeper = account.bookkeeper.to_string();
    let buyer = account.buyer.to_string();
    let seller = account.seller.to_string();
    let auction_house = account.auction_house.to_string();
    let metadata = account.metadata.to_string();
    let token_size = account.token_size as i64;
    let price = account.price as i64;
    let bump = account.bump as i32;
    let created_at_on_chain = account.created_at;
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &bookkeeper,
                &buyer,
                &seller,
                &auction_house,
                &metadata,
                &token_size,
                &price,
                &bump,
                &created_at_on_chain,
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

use mpl_auction_house::AuctionHouse;
use tokio_postgres::Client;
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("auction_house_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &AuctionHouse,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();
    let auction_house_fee_account = account.auction_house_fee_account.to_string();
    let auction_house_treasury = account.auction_house_treasury.to_string();
    let treasury_withdrawal_destination = account.treasury_withdrawal_destination.to_string();
    let fee_withdrawal_destination = account.fee_withdrawal_destination.to_string();
    let treasury_mint = account.treasury_mint.to_string();
    let authority = account.authority.to_string();
    let creator = account.creator.to_string();
    let bump = account.bump as i32;
    let treasury_bump = account.treasury_bump as i32;
    let fee_payer_bump = account.fee_payer_bump as i32;
    let seller_fee_basis_points = account.seller_fee_basis_points as i32;
    let requires_sign_off = account.requires_sign_off;
    let can_change_sale_price = account.can_change_sale_price;
    let slot = slot as i64;
    let write_version = write_version as i64;

    let result = client
        .query_one(
            UPSERT_QUERY,
            &[
                &id,
                &auction_house_fee_account,
                &auction_house_treasury,
                &treasury_withdrawal_destination,
                &fee_withdrawal_destination,
                &treasury_mint,
                &authority,
                &creator,
                &bump,
                &treasury_bump,
                &fee_payer_bump,
                &seller_fee_basis_points,
                &requires_sign_off,
                &can_change_sale_price,
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

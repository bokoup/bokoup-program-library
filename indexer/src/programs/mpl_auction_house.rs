use crate::AccountMessageData;
use anchor_lang::AccountDeserialize;
use bpl_api_data::queries::mpl_auction_house::{
    auction_house, bid_receipt, listing_receipt, purchase_receipt,
};
pub use mpl_auction_house::{
    receipt::{
        BidReceipt, ListingReceipt, PurchaseReceipt, BID_RECEIPT_SIZE, LISTING_RECEIPT_SIZE,
        PURCHASE_RECEIPT_SIZE,
    },
    AuctionHouse, AUCTION_HOUSE_SIZE, ID,
};

#[tracing::instrument(skip_all)]
async fn process_auction_house<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match AuctionHouse::try_deserialize(buf) {
        Ok(ref account) => {
            auction_house::upsert(pg_client, key, account, slot, write_version).await
        }
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_bid_receipt<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match BidReceipt::try_deserialize(buf) {
        Ok(ref account) => bid_receipt::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_listing_receipt<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match ListingReceipt::try_deserialize(buf) {
        Ok(ref account) => {
            listing_receipt::upsert(pg_client, key, account, slot, write_version).await
        }
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_purchase_receipt<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match PurchaseReceipt::try_deserialize(buf) {
        Ok(ref account) => {
            purchase_receipt::upsert(pg_client, key, account, slot, write_version).await
        }
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

pub async fn process<'a>(pg_client: deadpool_postgres::Object, message: AccountMessageData<'a>) {
    let key = message.account.pubkey.as_ref();
    let mut buf = message.account.data.as_ref();
    let slot = message.slot;
    let write_version = message.account.write_version;

    match buf.len() {
        AUCTION_HOUSE_SIZE => {
            process_auction_house(&pg_client, key, &mut buf, slot, write_version).await
        }
        BID_RECEIPT_SIZE => {
            process_bid_receipt(&pg_client, key, &mut buf, slot, write_version).await
        }
        LISTING_RECEIPT_SIZE => {
            process_listing_receipt(&pg_client, key, &mut buf, slot, write_version).await
        }
        PURCHASE_RECEIPT_SIZE => {
            process_purchase_receipt(&pg_client, key, &mut buf, slot, write_version).await
        }
        _ => (),
    }
}

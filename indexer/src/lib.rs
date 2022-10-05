use programs::{bpl_token_metadata, mpl_auction_house, mpl_token_metadata, spl_token};
use serde::{Deserialize, Serialize};

pub mod programs;

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageData<'a> {
    #[serde(borrow)]
    pub account: AccountData<'a>,
    pub slot: u64,
    pub is_startup: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountData<'a> {
    #[serde(with = "serde_bytes")]
    pub pubkey: &'a [u8],
    pub lamports: u64,
    #[serde(with = "serde_bytes")]
    pub owner: &'a [u8],
    pub executable: bool,
    pub rent_epoch: u64,
    #[serde(with = "serde_bytes")]
    pub data: &'a [u8],
    pub write_version: u64,
}

pub async fn process<'a>(pg_client: deadpool_postgres::Object, message: MessageData<'a>) {
    if message.account.owner == bpl_token_metadata::ID.as_ref() {
        bpl_token_metadata::process(pg_client, message).await
    } else if message.account.owner == mpl_auction_house::ID.as_ref() {
        mpl_auction_house::process(pg_client, message).await
    } else if message.account.owner == mpl_token_metadata::ID.as_ref() {
        mpl_token_metadata::process(pg_client, message).await
    } else if message.account.owner == spl_token::ID.as_ref() {
        spl_token::process(pg_client, message).await
    };
}

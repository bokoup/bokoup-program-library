use anchor_lang::prelude::Pubkey;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;

pub mod programs;

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountMessageData<'a> {
    #[serde(borrow)]
    pub account: AccountData<'a>,
    pub slot: u64,
    pub is_startup: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionMessageData {
    pub signature: Signature,
    pub program_id: Pubkey,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub slot: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageData<'a> {
    #[serde(borrow)]
    Account(AccountMessageData<'a>),
    Transaction(TransactionMessageData),
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
    match message {
        MessageData::Account(message) => {
            if message.account.owner == programs::bpl_token_metadata::ID.as_ref() {
                programs::bpl_token_metadata::process(pg_client, message).await
            } else if message.account.owner == programs::mpl_auction_house::ID.as_ref() {
                programs::mpl_auction_house::process(pg_client, message).await
            } else if message.account.owner == programs::mpl_token_metadata::ID.as_ref() {
                programs::mpl_token_metadata::process(pg_client, message).await
            } else if message.account.owner == programs::spl_token::ID.as_ref() {
                programs::spl_token::process(pg_client, message).await
            };
        }
        MessageData::Transaction(message) => {
            if message.program_id == programs::bpl_token_metadata::ID {
                programs::bpl_token_metadata::process_transaction(pg_client, message).await
            }
        }
    }
}

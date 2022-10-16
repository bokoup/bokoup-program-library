use crate::AccountMessageData;
use anchor_lang::AccountDeserialize;
pub use anchor_spl::token::{spl_token::ID, Mint, TokenAccount};
use bpl_api_data::{
    queries::spl_token::{mint, token_account},
    Client,
};

#[tracing::instrument(skip_all)]
async fn process_mint<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match Mint::try_deserialize(buf) {
        Ok(ref account) => mint::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

#[tracing::instrument(skip_all)]
async fn process_token_account<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match TokenAccount::try_deserialize(buf) {
        Ok(ref account) => {
            token_account::upsert(pg_client, key, account, slot, write_version).await
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
        Mint::LEN => process_mint(&pg_client, key, &mut buf, slot, write_version).await,
        TokenAccount::LEN => {
            process_token_account(&pg_client, key, &mut buf, slot, write_version).await
        }
        _ => (),
    }
}

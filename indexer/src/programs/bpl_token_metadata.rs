use crate::MessageData;
use anchor_lang::AccountDeserialize;
use bpl_hasura::{queries::bpl_token_metadata::promo, Client};
pub use bpl_token_metadata::{state::Promo, ID};

#[tracing::instrument(skip_all)]
async fn process_promo<'a>(
    pg_client: &Client,
    key: &[u8],
    buf: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match Promo::try_deserialize(buf) {
        Ok(ref account) => promo::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

pub async fn process<'a>(pg_client: deadpool_postgres::Object, message: MessageData<'a>) {
    let key = message.account.pubkey.as_ref();
    let mut buf = message.account.data.as_ref();
    let slot = message.slot;
    let write_version = message.account.write_version;

    match buf.len() {
        Promo::LEN => process_promo(&pg_client, key, &mut buf, slot, write_version).await,
        _ => (),
    }
}

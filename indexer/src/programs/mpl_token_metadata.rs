use crate::AccountMessageData;
use bpl_api_data::queries::mpl_token_metadata::metadata;
pub use mpl_token_metadata::{
    state::{Key::MetadataV1, Metadata, MAX_METADATA_LEN},
    utils::try_from_slice_checked,
    ID,
};

#[tracing::instrument(skip_all)]
async fn process_metadata<'a>(
    pg_client: &bpl_api_data::Client,
    key: &[u8],
    data: &mut &[u8],
    slot: u64,
    write_version: u64,
) {
    match try_from_slice_checked(data, MetadataV1, MAX_METADATA_LEN) {
        Ok(ref account) => metadata::upsert(pg_client, key, account, slot, write_version).await,
        Err(error) => {
            tracing::error!(id = bs58::encode(key).into_string(), ?error)
        }
    }
}

pub async fn process<'a>(pg_client: deadpool_postgres::Object, message: AccountMessageData<'a>) {
    let key = message.account.pubkey.as_ref();
    let mut data = message.account.data.as_ref();
    let slot = message.slot;
    let write_version = message.account.write_version;

    match data.len() {
        MAX_METADATA_LEN => process_metadata(&pg_client, key, &mut data, slot, write_version).await,
        _ => (),
    }
}

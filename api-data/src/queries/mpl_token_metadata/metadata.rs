use crate::queries::mpl_token_metadata::creator;
use futures::future::try_join;
use mpl_token_metadata::state::{Key, Metadata, TokenStandard, UseMethod};
use tokio_postgres::{types::Json, Client};
use tracing::{error, info};

const UPSERT_QUERY: &str = include_str!("metadata_upsert.sql");

#[tracing::instrument(skip_all)]
pub async fn upsert(
    client: &Client,
    key: &[u8],
    account: &Metadata,
    slot: u64,
    write_version: u64,
) {
    let id = bs58::encode(key).into_string();

    let mpl_key = match account.key {
        Key::Uninitialized => "Uninitialized",
        Key::EditionV1 => "EditionV1",
        Key::MasterEditionV1 => "MasterEditionV1",
        Key::ReservationListV1 => "ReservationListV1",
        Key::MetadataV1 => "MetadataV1",
        Key::ReservationListV2 => "ReservationListV2",
        Key::MasterEditionV2 => "MasterEditionV2",
        Key::EditionMarker => "EditionMarker",
        Key::UseAuthorityRecord => "UseAuthorityRecord",
        Key::CollectionAuthorityRecord => "CollectionAuthorityRecord",
    };

    let update_authority = account.update_authority.to_string();
    let mint = account.mint.to_string();
    let name = account.data.name.trim_matches(char::from(0)).to_string();
    let symbol = account.data.symbol.trim_matches(char::from(0)).to_string();
    let uri = account.data.uri.trim_matches(char::from(0)).to_string();
    let seller_fee_basis_points = account.data.seller_fee_basis_points as i32;
    let primary_sale_happened = account.primary_sale_happened;
    let is_mutable = account.is_mutable;
    let edition_nonce = account.edition_nonce.map(|v| v as i32);

    let token_standard = account.token_standard.as_ref().map(|v| match v {
        TokenStandard::NonFungible => "NonFungible",
        TokenStandard::FungibleAsset => "FungibleAsset",
        TokenStandard::Fungible => "Fungible",
        TokenStandard::NonFungibleEdition => "NonFungibleEdition",
    });

    let (collection_key, collection_verified) =
        if let Some(collection) = account.collection.as_ref() {
            (Some(collection.key.to_string()), Some(collection.verified))
        } else {
            (None, None)
        };

    let (uses_use_method, uses_remaining, uses_total) = if let Some(uses) = account.uses.as_ref() {
        let use_method = match uses.use_method {
            UseMethod::Burn => "Burn",
            UseMethod::Multiple => "Multiple",
            UseMethod::Single => "Single",
        };
        (
            Some(use_method),
            Some(uses.remaining as i64),
            Some(uses.total as i64),
        )
    } else {
        (None, None, None)
    };

    let slot = slot as i64;
    let write_version = write_version as i64;

    let metadata_json: Option<serde_json::Value> = if let Ok(response) = reqwest::get(&uri).await {
        if let Ok(result) = response.json::<serde_json::Value>().await {
            Some(result)
        } else {
            None
        }
    } else {
        None
    };

    let result = try_join(
        client.query_one(
            UPSERT_QUERY,
            &[
                &id,
                &mpl_key,
                &update_authority,
                &mint,
                &name,
                &symbol,
                &uri,
                &Json::<Option<serde_json::Value>>(metadata_json),
                &seller_fee_basis_points,
                &primary_sale_happened,
                &is_mutable,
                &edition_nonce,
                &token_standard,
                &collection_key,
                &collection_verified,
                &uses_use_method,
                &uses_remaining,
                &uses_total,
                &slot,
                &write_version,
            ],
        ),
        creator::upsert(
            client,
            key,
            account.data.creators.as_ref(),
            slot,
            write_version,
        ),
    )
    .await;
    match result {
        Ok((row, _)) => {
            let insert = row.get::<usize, Option<bool>>(0).unwrap();
            info!(id = id.as_str(), insert);
        }
        Err(ref error) => {
            error!(id = id.as_str(), ?error);
        }
    }
}

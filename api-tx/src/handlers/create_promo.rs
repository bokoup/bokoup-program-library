use crate::{
    error::AppError,
    utils::solana::{create_create_promo_instruction, SendTransResultObject},
    State,
};
use axum::{extract::Multipart, Extension, Json};
use bundlr_sdk::tags::Tag;
use serde_json::{json, Value};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use std::sync::Arc;

pub const LABEL: &str = "bokoup";
pub const ICON: &str = "https://arweave.net/wrKmRzr2KhH92c1iyFeUqkB-AHjYlE7Md7U5rK4qA8M";

pub async fn handler(
    mut multipart: Multipart,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Value>, AppError> {
    // Parse data - two parts - json data and image.
    let mut json_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "json-data" {
            let json_string = field.text().await.map_err(|_| {
                AppError::CreatePromoRequestError("trait attribute value not valid".to_string())
            })?;
            Ok(serde_json::from_str::<Value>(&json_string)?)
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::CreatePromoRequestError(
            "request had no parts".to_string(),
        ))
    }?;

    let image_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "image" {
            let content_type = field.content_type().map(ToString::to_string).ok_or(
                AppError::CreatePromoRequestError("failed to read image content type".to_string()),
            )?;
            let image_bytes = field.bytes().await.map_err(|_| {
                AppError::CreatePromoRequestError("failed to read image bytes".to_string())
            })?;
            Ok((image_bytes, content_type))
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::CreatePromoRequestError(
            "request only had one part".to_string(),
        ))
    }?;

    // Upload image to Arweave.
    let tx = state.bundlr.create_transaction_with_tags(
        image_data.0.to_vec(),
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), image_data.1.clone()),
        ],
    );

    // Get id of uploaded image and add to metdata json.
    let response = state.bundlr.send_transaction(tx).await?;
    let image_id = response
        .as_object()
        .ok_or(AppError::CreatePromoRequestError(
            "bundlr respsons should be an object".to_string(),
        ))?;

    let image_url = format!(
        "https://arweave.net/{}",
        image_id["id"]
            .as_str()
            .ok_or(AppError::CreatePromoRequestError(
                "id field should exist in bundlr response".to_string(),
            ))?
    );

    let json_data_obj = json_data
        .as_object_mut()
        .ok_or(AppError::CreatePromoRequestError(
            "json data part should be an object".to_string(),
        ))?;

    json_data_obj.insert("image".to_string(), image_url.clone().into());

    json_data_obj.insert(
        "properties".to_string(),
        json!({
            "files": [{
                "uri": image_url,
                "type": image_data.1
            }],
            "category": "image"
        }),
    );

    // Upload json metadata to Arweave and get id back for inclusion in creation of on chain Promo.
    let tx = state.bundlr.create_transaction_with_tags(
        serde_json::to_vec(json_data_obj)?,
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), "application/json".to_string()),
        ],
    );

    let response = state.bundlr.send_transaction(tx).await?;
    let metadata_id = response
        .as_object()
        .ok_or(AppError::CreatePromoRequestError(
            "bundlr respsons should be an object".to_string(),
        ))?;

    let uri = format!(
        "https://arweave.net/{}",
        metadata_id["id"]
            .as_str()
            .ok_or(AppError::CreatePromoRequestError(
                "id field should exist in bundlr response".to_string(),
            ))?
    );

    tracing::debug!(uri = &uri);

    let mint_keypair = Keypair::new();

    let name = json_data_obj["name"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "name field should exist".to_string(),
        ))?
        .to_string();

    let symbol = json_data_obj["symbol"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "symbol field should exist".to_string(),
        ))?
        .to_string();

    // Return max_mint and max_burn if attributes exists in json data and either exist in attributes, otherwise None
    let (max_mint, max_burn) = if let Some(attributes) = json_data_obj["attributes"].as_array() {
        let max_mint: Option<u32> = attributes
            .iter()
            .filter_map(|a| {
                let attribute = a.as_object()?;
                if let Some(value) = attribute.get("max_mint") {
                    value.as_u64()
                } else {
                    None
                }
            })
            .collect::<Vec<u64>>()
            .first()
            .map(|v| v.clone() as u32);

        let max_burn: Option<u32> = attributes
            .iter()
            .filter_map(|a| {
                let attribute = a.as_object()?;
                if let Some(value) = attribute.get("max_burn") {
                    value.as_u64()
                } else {
                    None
                }
            })
            .collect::<Vec<u64>>()
            .first()
            .map(|v| v.clone() as u32);

        (max_mint, max_burn)
    } else {
        (None, None)
    };

    let ix = create_create_promo_instruction(
        state.promo_owner.pubkey(),
        mint_keypair.pubkey(),
        state.platform.pubkey(),
        name,
        symbol,
        uri,
        max_mint,
        max_burn,
        true,
        None,
    )?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&state.promo_owner.pubkey()));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.sign(&[&state.promo_owner, &mint_keypair], latest_blockhash);

    let serialized = bincode::serialize(&tx)?;
    let tx_str = base64::encode(serialized);
    let response = state.solana.post_transaction_test(&tx_str).await?;

    tracing::debug!(response = format!("{:?}", response));

    Ok(Json(response))
}

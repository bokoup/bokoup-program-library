use crate::{error::AppError, State};
use bundlr_sdk::tags::Tag;
use serde_json::{json, Map, Value};
use std::sync::Arc;

pub async fn upload_image(
    image_data: (Vec<u8>, String),
    state: Arc<State>,
) -> Result<(String, String, Arc<State>), AppError> {
    // Upload image to Arweave.
    let tx = state.bundlr.create_transaction_with_tags(
        image_data.0,
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

    tracing::debug!(image_url = &image_url);

    Ok((image_url, image_data.1, state))
}

pub async fn upload_metadata_json(
    metadata_data_obj: &mut Map<String, Value>,
    image_url: String,
    content_type: String,
    state: Arc<State>,
) -> Result<(String, Arc<State>), AppError> {
    metadata_data_obj.insert("image".to_string(), image_url.clone().into());

    metadata_data_obj.insert(
        "properties".to_string(),
        json!({
            "files": [{
                "uri": image_url,
                "type": content_type
            }],
            "category": "image"
        }),
    );

    // Upload json metadata to Arweave and get id back for inclusion in creation of on chain Promo.
    let tx = state.bundlr.create_transaction_with_tags(
        serde_json::to_vec(metadata_data_obj)?,
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

    tracing::debug!(metadata_json_uri = &uri);

    Ok((uri, state))
}

use crate::{error::AppError, State};
use axum::{extract::Multipart, Extension, Json};
use bundlr_sdk::tags::Tag;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

pub const LABEL: &str = "bokoup";
pub const ICON: &str = "https://arweave.net/wrKmRzr2KhH92c1iyFeUqkB-AHjYlE7Md7U5rK4qA8M";

pub async fn handler(
    mut multipart: Multipart,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ResponseData>, AppError> {
    let mut json_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "json-data" {
            let json_string = field.text().await.map_err(|_| {
                AppError::CreatePromoRequestError("trait attribute value not valid".to_string())
            })?;
            Some(serde_json::from_str::<Value>(&json_string)?)
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        None
    }
    .expect("json data should be a value");
    let image_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "image" {
            let content_type = field.content_type().map(ToString::to_string).ok_or(
                AppError::CreatePromoRequestError("failed to read image content type".to_string()),
            )?;
            let image_bytes = field.bytes().await.map_err(|_| {
                AppError::CreatePromoRequestError("failed to read image bytes".to_string())
            })?;
            Some((image_bytes, content_type))
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        None
    }
    .expect("image data should be uploaded");

    let tx = state.bundlr.create_transaction_with_tags(
        image_data.0.to_vec(),
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), image_data.1.clone()),
        ],
    );

    let mut response = state.bundlr.send_transaction(tx).await?;
    let image_id = response.as_object_mut().unwrap();

    let image_url = format!("https://arweave.net/{}", image_id["id"].as_str().unwrap());
    let json_data_obj = json_data.as_object_mut().expect("should be an object");

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

    let tx = state.bundlr.create_transaction_with_tags(
        serde_json::to_vec(json_data_obj)?,
        vec![
            Tag::new("User-Agent".into(), "bokoup".into()),
            Tag::new("Content-Type".into(), "application/json".to_string()),
        ],
    );

    let response = state.bundlr.send_transaction(tx).await?;
    let metadata_id = response.as_object().unwrap();

    let metadata_url = format!(
        "https://arweave.net/{}",
        metadata_id["id"].as_str().unwrap()
    );

    // let file_name = field.file_name().unwrap_or("none").to_string();
    // let content_type = field.content_type().unwrap().to_string();
    // let data = field.bytes().await.unwrap();
    // let value = field.value().await.unwrap();

    tracing::debug!(metadata_url);

    Ok(Json(ResponseData {
        label: LABEL.to_string(),
        icon: ICON.to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    pub label: String,
    pub icon: String,
}

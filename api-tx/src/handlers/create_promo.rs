use std::str::FromStr;

use crate::error::AppError;
use axum::{extract::Multipart, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const LABEL: &str = "bokoup";
pub const ICON: &str = "https://arweave.net/wrKmRzr2KhH92c1iyFeUqkB-AHjYlE7Md7U5rK4qA8M";

pub async fn handler(mut multipart: Multipart) -> Result<Json<ResponseData>, AppError> {
    let text_fields: Vec<String> = vec!["name", "symbol", "description"]
        .iter()
        .map(ToString::to_string)
        .collect();
    let number_fields = vec!["max_mint".to_string(), "max_burn".to_string()];

    let fields = [
        "name",
        "symbol",
        "description",
        "attributes",
        "collection",
        "max_mint",
        "max_burn",
        "image",
    ];

    let mut attr_trait = "".to_string();
    let mut attr_value = "".to_string();
    let attributes: Vec<Value> = Vec::new();

    let json_data = if let Some(field) = multipart.next_field().await.unwrap() {
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
    };
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
    };

    // let file_name = field.file_name().unwrap_or("none").to_string();
    // let content_type = field.content_type().unwrap().to_string();
    // let data = field.bytes().await.unwrap();
    // let value = field.value().await.unwrap();

    // tracing::debug!(name, file_name, content_type, data_len = data.len());

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

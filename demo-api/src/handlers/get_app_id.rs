use crate::error::AppError;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const LABEL: &str = "Demo";
pub const ICON: &str = "https://arweave.net/47oYXF2a6izPAaimwCalKShQ_YXqydX3fjm0cjWLbts";

pub async fn handler(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ResponseData>, AppError> {
    log::debug!("get_app_id: {:?}", params);
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

use crate::error::AppError;
use axum::Json;
use serde::{Deserialize, Serialize};

pub const LABEL: &str = "Demo";
pub const ICON: &str = "https://arweave.net/47oYXF2a6izPAaimwCalKShQ_YXqydX3fjm0cjWLbts";

pub async fn handler() -> Result<Json<ResponseData>, AppError> {
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

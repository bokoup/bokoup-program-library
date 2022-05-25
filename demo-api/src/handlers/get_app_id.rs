use crate::{error::AppError, handlers::Params};
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};

pub const LABEL: &str = "Demo";
pub const ICON: &str = "https://arweave.net/47oYXF2a6izPAaimwCalKShQ_YXqydX3fjm0cjWLbts";

pub async fn handler(
    Path(Params {
        mint_string,
        promo_name,
    }): Path<Params>,
) -> Result<Json<ResponseData>, AppError> {
    tracing::debug!(
        "get_app_id:mint_string: {}, promo_name: {}",
        mint_string,
        promo_name
    );
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

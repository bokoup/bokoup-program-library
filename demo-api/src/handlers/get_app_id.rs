use crate::{error::AppError, handlers::Params};
use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};

pub const LABEL: &str = "bokoup";
pub const ICON: &str = "https://arweave.net/wrKmRzr2KhH92c1iyFeUqkB-AHjYlE7Md7U5rK4qA8M";

pub async fn handler(
    Path(Params {
        mint_string,
        message,
        memo,
    }): Path<Params>,
) -> Result<Json<ResponseData>, AppError> {
    tracing::debug!(mint_string = mint_string, message = message, memo = memo);
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

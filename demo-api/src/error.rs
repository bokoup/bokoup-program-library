use serde_json::json;
use thiserror::Error;

use axum::{
    body::BoxBody,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

/// Errors propagated by library functions.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("deep hash item is not a list")]
    GenericError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        let status = match self {
            AppError::GenericError => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status, body).into_response()
    }
}

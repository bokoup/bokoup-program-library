use anchor_lang::solana_program::pubkey::ParsePubkeyError;
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
    #[error("generic error")]
    GenericError,
    #[error("bincode: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("bincode: {0}")]
    PubkeyError(#[from] ParsePubkeyError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        let status = match self {
            AppError::GenericError => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status, body).into_response()
    }
}

use anchor_lang::solana_program::pubkey::ParsePubkeyError;
use serde_json::json;
use solana_sdk::signature::SignerError;
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
    #[error("generic error: {0}")]
    GenericError(String),
    #[error("bincode: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("bundlr: {0}")]
    BundlrError(#[from] bundlr_sdk::error::BundlrError),
    #[error("create promo request: {0}")]
    CreatePromoRequestError(String),
    #[error("data query error")]
    DataQueryError,
    #[error("pubkey error")]
    PubkeyError(#[from] ParsePubkeyError),
    #[error("payer not in members")]
    PayerNotInMembers,
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("serde json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("signing error")]
    SignError(#[from] SignerError),
    #[error("error confirming posted solana transaction: {0}")]
    SolanaGetError(reqwest::Error),
    #[error("solana hash parse {0}")]
    SolanaHashParse(#[from] solana_sdk::hash::ParseHashError),
    #[error("error posting solana transaction: {0}")]
    SolanaPostError(String),
    #[error("error posting to clover: {0}")]
    CloverPostError(reqwest::Error),
    #[error("error parsing url: {0}")]
    UrlParseError(url::ParseError),
    #[error("error decoding url encoded string: {0}")]
    UrlDecodeError(std::string::FromUtf8Error),
    #[error("clover status not OK: {0}")]
    StatusNotOK(reqwest::StatusCode),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<BoxBody> {
        let status = match self {
            AppError::GenericError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };

        let body = Json(json!({
            "error": self.to_string(),
        }));
        (status, body).into_response()
    }
}

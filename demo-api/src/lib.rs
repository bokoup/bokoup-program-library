use anchor_lang::prelude::Pubkey;
use axum::{
    error_handling::HandleErrorLayer,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use solana_sdk::transaction::Transaction;
use std::{borrow::Cow, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

pub mod error;
use error::AppError;

pub mod utils;
use utils::create_transfer_promo_instruction;

pub fn create_app() -> Router {
    Router::new()
        .route("/", get(get_app_id))
        .route("/promo", post(get_transfer_promo_tx))
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
}

async fn get_app_id() -> Result<Json<AppId>, AppError> {
    Ok(Json(AppId {
        label: "Bokoup App".to_string(),
        icon: "https://arweave.net/47oYXF2a6izPAaimwCalKShQ_YXqydX3fjm0cjWLbts".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AppId {
    pub label: String,
    pub icon: String,
}

async fn get_transfer_promo_tx(
    Json(payload): Json<TransferPromo>,
) -> Result<Json<ResponseTx>, AppError> {
    let instruction = create_transfer_promo_instruction(payload.account, payload.mint).await?;

    let tx = Transaction::new_with_payer(&[instruction], Some(&payload.account));
    let serialized = bincode::serialize(&tx)?;
    let transaction = base64::encode(serialized);

    Ok(Json(ResponseTx {
        transaction,
        message: "You've got a promo!".to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TransferPromo {
    account: Pubkey,
    mint: Pubkey,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResponseTx {
    transaction: String,
    message: String,
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}

#[cfg(test)]
pub mod test {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_app_id() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: AppId = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            parsed_response,
            AppId {
                label: "Bokoup App".to_string(),
                icon: "https://arweave.net/47oYXF2a6izPAaimwCalKShQ_YXqydX3fjm0cjWLbts".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_transfer_promo_tx() {
        let app = create_app();
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let transfer_promo = TransferPromo { account, mint };

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/promo")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&transfer_promo).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: ResponseTx = serde_json::from_slice(&body).unwrap();

        let instruction = create_transfer_promo_instruction(account, mint)
            .await
            .unwrap();

        let tx = Transaction::new_with_payer(&[instruction], Some(&account));
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            ResponseTx {
                transaction,
                message: "You've got a promo!".to_string(),
            }
        );
    }
}

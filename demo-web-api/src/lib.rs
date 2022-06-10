use axum::{
    error_handling::HandleErrorLayer,
    handler::Handler,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use handlers::*;
use solana_sdk::{
    signature::read_keypair_file,
    signer::{keypair::Keypair, Signer},
};
use std::{borrow::Cow, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use url::Url;
use utils::Solana;

pub mod error;
pub mod handlers;
pub mod utils;

pub const SOL_URL: &str = "https://api.devnet.solana.com/";

pub struct State {
    pub promo_owner: Keypair,
    pub solana: Solana,
}

impl Default for State {
    fn default() -> Self {
        Self {
            promo_owner: read_keypair_file("/keys/promo_owner-keypair.json").unwrap(),
            solana: Solana {
                base_url: SOL_URL.parse::<Url>().unwrap(),
                client: reqwest::Client::new(),
            },
        }
    }
}

pub fn create_app() -> Router {
    let state = State::default();
    tracing::debug!("promo_owner: {}", state.promo_owner.pubkey().to_string());

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    Router::new()
        .route("/promo/:mint_string/:promo_name", get(get_app_id::handler))
        .route(
            "/promo/:mint_string/:promo_name",
            post(get_mint_promo_tx::handler.layer(AddExtensionLayer::new(Arc::new(state)))),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
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
    use solana_sdk::transaction::Transaction;
    use tower::ServiceExt;
    use utils::create_transfer_promo_instruction;

    #[tokio::test]
    async fn test_app_id() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/promo/ding/dong")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: get_app_id::ResponseData = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            parsed_response,
            get_app_id::ResponseData {
                label: get_app_id::LABEL.to_string(),
                icon: get_app_id::ICON.to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_transfer_promo_tx() {
        let state = State::default();
        let app = create_app();
        let wallet = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: wallet.to_string(),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/promo/{}/ding", mint.to_string()))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: get_mint_promo_tx::ResponseData =
            serde_json::from_slice(&body).unwrap();

        let instruction =
            create_transfer_promo_instruction(wallet, mint, state.promo_owner.pubkey())
                .await
                .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&wallet));
        tx.try_partial_sign(
            &[&state.promo_owner],
            state.solana.get_latest_blockhash().await.unwrap(),
        )
        .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            get_mint_promo_tx::ResponseData {
                transaction,
                message: "Approve to receive ding.".to_string(),
            }
        );
    }
}

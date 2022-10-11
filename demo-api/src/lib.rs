use anchor_lang::prelude::Pubkey;
use axum::{
    error_handling::HandleErrorLayer,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use handlers::*;
use solana_sdk::{signature::read_keypair_file, signer::keypair::Keypair};
use std::{borrow::Cow, str::FromStr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    add_extension::AddExtensionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use url::Url;
use utils::{clover::Clover, solana::Solana};

pub mod error;
pub mod handlers;
pub mod utils;

pub const SOL_URL: &str = "https://api.devnet.solana.com/";
pub const PLATFORM_ADDRESS: &str = "2R7GkXvQQS4iHptUvQMhDvRSNXL8tAuuASNvCYgz3GQW";
pub const CLOVER_URL: &str = "https://sandbox.dev.clover.com/v3/apps/";
pub const CLOVER_APP_ID: &str = "MAC8DQKWCCB1R";

pub struct State {
    pub promo_owner: Keypair,
    pub platform: Pubkey,
    pub solana: Solana,
    pub clover: Clover,
}

type SharedState = Arc<State>;

impl Default for State {
    fn default() -> Self {
        Self {
            promo_owner: read_keypair_file("/keys/promo_owner-keypair.json").unwrap(),
            platform: Pubkey::from_str(PLATFORM_ADDRESS).unwrap(),
            solana: Solana {
                base_url: SOL_URL.parse::<Url>().unwrap(),
                client: reqwest::Client::new(),
            },
            clover: Clover {
                base_url: CLOVER_URL
                    .parse::<Url>()
                    .unwrap()
                    .join(format!("{CLOVER_APP_ID}/").as_str())
                    .unwrap(),
                client: reqwest::Client::new(),
            },
        }
    }
}

pub fn create_app() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_origin(Any);

    Router::new()
        .route(
            "/promo/mint/:mint_string/:message/:memo",
            get(get_app_id::handler).post(get_mint_promo_tx::handler),
        )
        .route(
            "/promo/delegate/:mint_string/:message/:memo",
            get(get_app_id::handler).post(get_delegate_promo_tx::handler),
        )
        .route(
            "/promo/burn-delegated/:mint_string/:message/:memo",
            get(get_app_id::handler).post(get_burn_delegated_promo_tx::handler),
        )
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .layer(AddExtensionLayer::new(SharedState::default()))
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
    use solana_sdk::{signer::Signer, transaction::Transaction};
    use tower::ServiceExt;
    use utils::solana::*;
    const MESSAGE: &str =
        "This is a really long message that tells you do do something with your $$!!";
    const MEMO: &str = "Please, listen to my memo.";

    #[tokio::test]
    async fn test_app_id() {
        tracing_subscriber::fmt::init();
        let app = create_app();
        let mint = Pubkey::new_unique();
        let message = urlencoding::encode(MESSAGE);
        let memo = urlencoding::encode(MEMO);
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/promo/mint/{}/{}/{}",
                        mint.to_string(),
                        message.into_owned(),
                        memo.into_owned()
                    ))
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
    async fn test_get_mint_promo_tx() {
        let state = State::default();
        let app = create_app();
        let token_owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: token_owner.to_string(),
        };
        let message = urlencoding::encode(MESSAGE);
        let memo = urlencoding::encode(MEMO);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/mint/{}/{}/{}",
                        mint.to_string(),
                        message.into_owned(),
                        memo.into_owned()
                    ))
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

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction =
            create_mint_promo_instruction(state.promo_owner.pubkey(), token_owner, mint)
                .await
                .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
        tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            get_mint_promo_tx::ResponseData {
                transaction,
                message: MESSAGE.to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_delegate_promo_tx() {
        dotenv::dotenv().ok();

        let state = State::default();
        let app = create_app();
        let token_owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: token_owner.to_string(),
        };

        let message = urlencoding::encode(MESSAGE);
        let memo = urlencoding::encode(MEMO);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/delegate/{}/{}/{}",
                        mint.to_string(),
                        message.into_owned(),
                        memo.into_owned()
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: get_burn_delegated_promo_tx::ResponseData =
            serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction =
            create_delegate_promo_instruction(state.promo_owner.pubkey(), token_owner, mint)
                .await
                .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
        tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            get_burn_delegated_promo_tx::ResponseData {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn test_get_burn_delegated_promo_tx() {
        dotenv::dotenv().ok();

        let state = State::default();
        let app = create_app();
        let token_owner = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let data = get_mint_promo_tx::Data {
            account: token_owner.to_string(),
        };

        let message = urlencoding::encode(MESSAGE);
        let memo = urlencoding::encode(MEMO);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!(
                        "/promo/burn-delegated/{}/{}/{}",
                        mint.to_string(),
                        message.into_owned(),
                        memo.into_owned()
                    ))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(serde_json::to_vec(&data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let parsed_response: get_burn_delegated_promo_tx::ResponseData =
            serde_json::from_slice(&body).unwrap();

        let txd: Transaction = bincode::deserialize(
            &base64::decode::<String>(parsed_response.transaction.clone()).unwrap(),
        )
        .unwrap();

        let instruction = create_burn_delegated_promo_instruction(
            state.promo_owner.pubkey(),
            token_owner,
            mint,
            state.platform,
        )
        .await
        .unwrap();

        let mut tx = Transaction::new_with_payer(&[instruction], Some(&state.promo_owner.pubkey()));
        tx.try_partial_sign(&[&state.promo_owner], txd.message.recent_blockhash)
            .unwrap();
        let serialized = bincode::serialize(&tx).unwrap();
        let transaction = base64::encode(serialized);

        assert_eq!(
            parsed_response,
            get_burn_delegated_promo_tx::ResponseData {
                transaction,
                message: MESSAGE.to_owned(),
            }
        );
    }
}

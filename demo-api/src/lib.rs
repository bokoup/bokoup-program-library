use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::{borrow::Cow, path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::{add_extension::AddExtensionLayer, trace::TraceLayer};

use error::AppError;
pub mod error;

pub fn create_app() -> Router {
    Router::new().route("/", get(root)).layer(
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

async fn root() -> &'static str {
    "Hello, demo!"
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
    use super::create_app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_hello_sol_ar() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, demo!");
    }
}

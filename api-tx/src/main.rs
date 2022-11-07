use bpl_api_tx::{create_app, utils::solana::Cluster};
use std::net::SocketAddr;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_api_tx=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = create_app(Cluster::Devnet);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

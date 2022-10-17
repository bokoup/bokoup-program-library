use bpl_api_tx::{create_app, utils::solana::SolanaUrl};
use std::net::SocketAddr;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = create_app(SolanaUrl::Localnet);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

use anchor_lang::prelude::Pubkey;
use bpl_api_tx::{create_app, parse_string_to_keypair, utils::solana::Cluster};
use clap::Parser;
use std::{net::SocketAddr, str::FromStr};
use tracing_subscriber::prelude::*;
use url::Url;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "l", value_parser)]
    cluster: Cluster,
    #[clap(
        long,
        default_value = "2R7GkXvQQS4iHptUvQMhDvRSNXL8tAuuASNvCYgz3GQW",
        value_parser
    )]
    platform: Pubkey,
    #[clap(long, env = "PLATFORM_SIGNER_KEYPAIR")]
    platform_signer: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_api_tx=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let platform_signer = parse_string_to_keypair(&args.platform_signer);

    let data_url: Url = Url::from_str(match args.cluster {
        Cluster::Devnet => "https://data.api.bokoup.dev/v1/graphql/",
        _ => "https://shining-sailfish-15.hasura.app/v1/graphql/",
    })
    .unwrap();

    let app = create_app(args.cluster, args.platform, platform_signer, data_url);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

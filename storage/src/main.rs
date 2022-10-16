use bundlr_sdk::{tags::Tag, Bundlr, Ed25519Signer};
use clap::{Parser, Subcommand};
use ed25519_dalek::Keypair;
use std::path::PathBuf;
use tracing_subscriber::prelude::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long)]
    quiet: bool,
    #[clap(long, default_value = "/home/caleb/.config/solana/devnet.json", value_parser = valid_file_path)]
    payer_path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    UploadString,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    if !cli.quiet {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_storage=trace".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let data = tokio::fs::read(cli.payer_path).await.unwrap();
    let bytes: Vec<u8> = serde_json::from_slice(&data).unwrap();
    let keypair = Keypair::from_bytes(&bytes).unwrap();
    // let keypair = read_keypair_file(cli.payer_path).unwrap();
    tracing::debug!(signer = bs58::encode(&keypair.public.as_ref()).into_string());
    let signer = Ed25519Signer::new(keypair);

    let bundlr = Bundlr::new(
        "https://node1.bundlr.network".to_string(),
        "solana".to_string(),
        "sol".to_string(),
        signer,
    );

    match &cli.command {
        Commands::UploadString => {
            let json_data = serde_json::json!({
                "name": "Test Promo",
                "symbol": "TEST",
                "description": "Bokoup test promotion.",
                "attributes": [
                    {  "trait_type": "discount",
                        "value": 10,
                    },
                    {
                        "trait_type": "expiration",
                        "value": "never",
                    },
                ],
                "collection": {
                    "name": "Test Merchant Promos",
                    "family": "Non-Fungible Offers"
                },
                "max_mint": 1000,
                "max_burn": 500
            });

            let tx = bundlr.create_transaction_with_tags(
                serde_json::to_vec(&json_data).unwrap(),
                vec![
                    Tag::new("hello".into(), "world".into()),
                    Tag::new("Content-Type".into(), "application/json".into()),
                ],
            );

            // Will return Err if not success
            match bundlr.send_transaction(tx).await {
                Ok(value) => println!("{}", value),
                Err(e) => println!("{}", e),
            }
        }
    }
}

// https://docs.bundlr.network/docs/client/examples/funding-your-account

// ====================
// Validators
// ====================

fn valid_file_path(path_str: &str) -> Result<PathBuf, String> {
    match path_str.parse::<PathBuf>() {
        Ok(p) => {
            if p.exists() {
                if p.is_file() {
                    Ok(p)
                } else {
                    Err(format!("path is not file."))
                }
            } else {
                Err(format!("path does not exist."))
            }
        }
        Err(_) => Err(format!("not a valid path.")),
    }
}

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
    #[clap(long, default_value = "~/.config/solana/devnet.json", value_parser = valid_file_path)]
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

    let bytes = tokio::fs::read(&cli.payer_path).await.unwrap();
    let payer_keypair = Keypair::from_bytes(&bytes).unwrap();

    if !cli.quiet {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_storage=trace".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    match &cli.command {
        Commands::UploadString => {
            let signer = Ed25519Signer::new(payer_keypair);
            tracing::info!(hello = "world")
        }
    }
}

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

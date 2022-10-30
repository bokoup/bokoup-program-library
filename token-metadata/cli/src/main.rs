use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_program,
    },
    Client, Cluster,
};
use bpl_token_metadata::{instruction, state::AdminSettings, utils};
use bundlr_sdk::{tags::Tag, Bundlr, Ed25519Signer};
use clap::{Parser, Subcommand};
use ed25519_dalek::Keypair as DalekKeypair;
use std::{path::PathBuf, rc::Rc};
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
    program_authority_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/platform-keypair.json", value_parser = valid_file_path)]
    platform_path: PathBuf,
    #[clap(long, default_value = "../../target/deploy/promo_owner-keypair.json", value_parser = valid_file_path)]
    promo_owner_path: PathBuf,
    #[clap(long, default_value_t = Cluster::Localnet, value_parser)]
    cluster: Cluster,
}

#[derive(Subcommand)]
enum Commands {
    Airdrop {
        #[clap(default_value_t = 2, value_parser)]
        sol: u64,
    },
    #[clap(about = "Placeholder demonstrating upload of json to arweave")]
    UploadString,
    #[clap(about = "Create or update admin settings account")]
    CreateAdminSettings {
        #[clap(long, default_value_t = 100_000_000, value_parser)]
        create_promo_lamports: u64,
        #[clap(long, default_value_t = 10_000_000, value_parser)]
        burn_promo_token_lamports: u64,
    },
    #[clap(about = "Tesing requesting data from graphql api")]
    TestGql,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    if !cli.quiet {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "bpl_token_metadata_cli=trace".into()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    let [program_authority_keypair, platform_keypair, promo_owner_keypair]: [Keypair; 3] = [
        &cli.program_authority_path,
        &cli.platform_path,
        &cli.promo_owner_path,
    ]
    .map(|p| read_keypair_file(p).expect("problem reading keypair file"));

    match &cli.command {
        Commands::Airdrop { sol } => {
            let program_authority = program_authority_keypair.pubkey();
            let rc_payer_keypair = Rc::new(program_authority_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());

            [
                &program_authority,
                &platform_keypair.pubkey(),
                &promo_owner_keypair.pubkey(),
            ]
            .iter()
            .for_each(|pubkey| {
                program
                    .rpc()
                    .request_airdrop(pubkey, sol * 1_000_000_000)
                    .unwrap();
            });

            Ok(())
        }
        Commands::CreateAdminSettings {
            create_promo_lamports,
            burn_promo_token_lamports,
        } => {
            let payer = program_authority_keypair.pubkey();
            let rc_payer_keypair = Rc::new(program_authority_keypair);
            let client = Client::new_with_options(
                cli.cluster,
                rc_payer_keypair,
                CommitmentConfig::confirmed(),
            );

            let program = client.program(bpl_token_metadata::id());
            let (admin_settings, _) = utils::find_admin_address();
            let program_data = utils::find_program_data_address();
            tracing::info!(program_data = program_data.to_string());

            let tx = program
                .request()
                .accounts(bpl_token_metadata::accounts::CreateAdminSettings {
                    payer,
                    admin_settings,
                    // program: bpl_token_metadata::id(),
                    // program_data,
                    system_program: system_program::ID,
                })
                .args(instruction::CreateAdminSettings {
                    data: AdminSettings {
                        platform: platform_keypair.pubkey(),
                        create_promo_lamports: create_promo_lamports.clone(),
                        burn_promo_token_lamports: burn_promo_token_lamports.clone(),
                    },
                })
                .send()?;
            let admin_settings_account: AdminSettings = program.account(admin_settings)?;
            tracing::info!(
                signature = tx.to_string(),
                admin_settings_account = format!("{:?}", admin_settings_account)
            );
            Ok(())
        }
        Commands::UploadString => {
            let data = tokio::fs::read(&cli.program_authority_path).await.unwrap();
            let bytes: Vec<u8> = serde_json::from_slice(&data).unwrap();
            let keypair = DalekKeypair::from_bytes(&bytes).unwrap();
            tracing::debug!(signer = bs58::encode(&keypair.public.as_ref()).into_string());
            let signer = Ed25519Signer::new(keypair);

            let bundlr = Bundlr::new(
                "https://node1.bundlr.network".to_string(),
                "solana".to_string(),
                "sol".to_string(),
                signer,
            );

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
                    Tag::new("User-Agent".into(), "bokoup".into()),
                    Tag::new("Content-Type".into(), "application/json".into()),
                ],
            );

            // Will return Err if not success
            match bundlr.send_transaction(tx).await {
                Ok(value) => println!("{}", value),
                Err(e) => println!("{}", e),
            }
            Ok(())
        }
        Commands::TestGql => {
            let client = reqwest::Client::new();
            let query = r#"
            query MintQuery($mint: String) {
                mint(where: {id: {_eq: $mint}}) {
                  promoObject {
                    groupObject {
                      id
                      seed
                    }
                  }
                }
              }
            "#;

            let result: serde_json::Value = client
                .post("https://shining-sailfish-15.hasura.app/v1/graphql/")
                .json(&serde_json::json!({ "query": query, "operationName": "MintQuery", "variables": {"mint": "4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM"}}))
                .send()
                .await?
                .json()
                .await?;

            println!("{}", result);
            Ok(())
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

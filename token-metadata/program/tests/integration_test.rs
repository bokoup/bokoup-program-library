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
use std::{path::PathBuf, rc::Rc, str::FromStr};
use tracing_subscriber::prelude::*;

#[tokio::test(flavor = "multi_thread")]
async fn anchor_tests() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| format!("{}=trace", module_path!())),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let anchor_toml = tokio::fs::read_to_string("../../Anchor.toml")
        .await
        .unwrap()
        .parse::<toml::Value>()
        .unwrap();

    let cluster = Cluster::from_str(
        anchor_toml
            .get("provider")
            .unwrap()
            .as_table()
            .unwrap()
            .get("cluster")
            .unwrap()
            .as_str()
            .unwrap(),
    )
    .unwrap();

    let program_authority_keypair = read_keypair_file("/home/caleb/.config/solana/devnet.json")
        .expect("problem reading keypair file");
    let platform_keypair = read_keypair_file("../../target/deploy/platform-keypair.json")
        .expect("problem reading keypair file");
    let promo_owner_keypair = read_keypair_file("../../target/deploy/promo_owner-keypair.json")
        .expect("problem reading keypair file");
    let group_member_keypair = read_keypair_file("../../target/deploy/group_member_1-keypair.json")
        .expect("problem reading keypair file");
    let group_seed = read_keypair_file("../../target/deploy/group_seed-keypair.json")
        .expect("problem reading keypair file");

    let program_authority = program_authority_keypair.pubkey();
    let rc_payer_keypair = Rc::new(program_authority_keypair);
    let client = Client::new_with_options(cluster, rc_payer_keypair, CommitmentConfig::confirmed());

    let program = client.program(bpl_token_metadata::id());

    [
        &program_authority,
        &platform_keypair.pubkey(),
        &promo_owner_keypair.pubkey(),
        &group_member_keypair.pubkey(),
    ]
    .iter()
    .for_each(|pubkey| {
        program
            .rpc()
            .request_airdrop(pubkey, 1_000_000_000)
            .unwrap();
    });
}

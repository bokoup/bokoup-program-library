use bpl_hasura::{get_client, reset, DatabaseURL};
use clap::{Parser, Subcommand};
use tokio_postgres::Error;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long)]
    quiet: bool,
    #[clap(long, arg_enum, default_value_t = DatabaseURL::default(), value_parser)]
    db_url: DatabaseURL,
}

#[derive(Subcommand)]
enum Commands {
    ResetSchema,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let password = std::env::var("PG_PASSWORD_LOCALNET").unwrap();
    let cli = Cli::parse();
    let client = get_client(&cli.db_url.url(password)).await?;

    if !cli.quiet {
        tracing_subscriber::fmt::init();
    }

    match &cli.command {
        Commands::ResetSchema => reset(&client).await,
    }
}

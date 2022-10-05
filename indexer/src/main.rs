use bpl_hasura::DatabaseURL;
use clap::Parser;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, arg_enum, default_value_t = DatabaseURL::default(), value_parser)]
    db_url: DatabaseURL,
    #[clap(long, default_value_t = 10, value_parser)]
    pg_pool_size: usize,
    #[clap(long, default_value = "localhost:4222", value_parser)]
    nats_url: String,
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "bpl_indexer=trace,bpl_hasura=trace");
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().ok();
    let args = Args::parse();

    // create db connection pool
    let pg_config = args.db_url.url().parse::<bpl_hasura::Config>().unwrap();
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, bpl_hasura::NoTls, mgr_config);
    let pg_pool = Pool::builder(mgr)
        .max_size(args.pg_pool_size)
        .build()
        .unwrap();
    tracing::info!(db_url = args.db_url.url(), pool_size = args.pg_pool_size);

    // connect to nats
    let nats_connection = nats::connect(args.nats_url.as_str()).unwrap();
    let sub = nats_connection.subscribe("update_account").unwrap();
    tracing::info!(
        nats_url = args.nats_url.as_str(),
        suscribed = "update_account"
    );

    // process messages
    for msg in sub.messages() {
        let message: bpl_indexer::MessageData = bincode::deserialize(msg.data.as_slice()).unwrap();
        let pg_client = pg_pool.get().await.unwrap();
        bpl_indexer::process(pg_client, message).await;
    }
}

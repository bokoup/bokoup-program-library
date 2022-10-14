use clap::ArgEnum;
pub use tokio_postgres::{types::FromSql, Client, Config, Error, NoTls};

pub mod queries;

const RESET_SCHEMA_SQL: &str = include_str!("./migrations/reset.sql");

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Default)]
pub enum DatabaseURL {
    #[default]
    Localnet,
    Devnet,
}

impl DatabaseURL {
    pub fn url(&self) -> String {
        match self {
            DatabaseURL::Localnet => std::env::var("PG_DATABASE_URL_LOCALNET").unwrap(),
            DatabaseURL::Devnet => std::env::var("PG_DATABASE_URL_DEVNET").unwrap(),
        }
    }
}

pub async fn get_client(db_url: &str) -> Result<Client, Error> {
    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}

#[tracing::instrument(skip_all)]
pub async fn reset(client: &mut Client) -> Result<(), Error> {
    client.batch_execute(RESET_SCHEMA_SQL).await?;
    let report = embedded::migrations::runner()
        .run_async(client)
        .await
        .unwrap();
    tracing::info!(applied_migrations = report.applied_migrations().len());
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn apply_migrations(client: &mut Client) -> Result<(), Error> {
    let report = embedded::migrations::runner()
        .run_async(client)
        .await
        .unwrap();
    tracing::info!(applied_migrations = report.applied_migrations().len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_spl::associated_token::get_associated_token_address;
    use bpl_token_metadata::state::Promo;
    use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
    use mpl_auction_house::{
        pda::{
            find_auction_house_address, find_bid_receipt_address, find_listing_receipt_address,
            find_purchase_receipt_address, find_trade_state_address,
        },
        receipt::{BidReceipt, ListingReceipt, PurchaseReceipt},
        AuctionHouse,
    };
    use mpl_token_metadata::{
        pda::find_metadata_account,
        state::{Collection, Creator, Data, Key, Metadata, TokenStandard, UseMethod, Uses},
    };

    use solana_sdk::signature::Signature;
    use spl_token::{
        solana_program::{program_option::COption, pubkey::Pubkey},
        state::{Account, AccountState, Mint},
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use tracing_subscriber;

    fn get_now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    // =============================
    // Accounts
    // =============================

    async fn it_upserts_mint(
        client: &Client,
        key: &[u8],
        account: &Mint,
        slot: u64,
        write_version: u64,
    ) {
        queries::spl_token::mint::upsert(client, key, account, slot, write_version).await;
        let row = client
            .query_one(
                "SELECT * FROM mint WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, i64>("supply"),
            account.supply as i64,
            "it_upserts_mint: supply failed"
        );
        assert_eq!(
            row.get::<&str, Option<String>>("mint_authority"),
            account.mint_authority.map(|k| k.to_string()).into(),
            "it_upserts_mint: mint_authority failed"
        );
    }

    async fn it_upserts_token_account(
        client: &Client,
        key: &[u8],
        account: &Account,
        slot: u64,
        write_version: u64,
    ) {
        queries::spl_token::token_account::upsert(client, key, account, slot, write_version).await;
        let row = client
            .query_one(
                "SELECT * FROM token_account WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, i64>("amount"),
            account.amount as i64,
            "it_upserts_token_account: amount failed"
        );
        assert_eq!(
            row.get::<&str, Option<String>>("close_authority"),
            account.close_authority.map(|k| k.to_string()).into(),
            "it_upserts_token_account: close_authority failed"
        );
    }

    async fn it_upserts_metadata(
        client: &Client,
        key: &[u8],
        metadata: &Metadata,
        slot: u64,
        write_version: u64,
    ) {
        queries::mpl_token_metadata::metadata::upsert(client, key, metadata, slot, write_version)
            .await;
        let row = client
            .query_one(
                "SELECT * FROM metadata WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, i32>("seller_fee_basis_points"),
            metadata.data.seller_fee_basis_points as i32,
            "it_upserts_metadata: seller_fee_basis_points"
        );
    }

    async fn it_upserts_promo(
        client: &Client,
        key: &[u8],
        promo: &Promo,
        slot: u64,
        write_version: u64,
    ) {
        queries::bpl_token_metadata::promo::upsert(client, key, promo, slot, write_version).await;
        let row = client
            .query_one(
                "SELECT * FROM promo WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, i32>("mint_count"),
            promo.mint_count as i32,
            "it_upserts_promo: mints"
        );
    }

    async fn it_upserts_auction_house(
        client: &Client,
        key: &[u8],
        auction_house: &AuctionHouse,
        slot: u64,
        write_version: u64,
    ) {
        queries::mpl_auction_house::auction_house::upsert(
            client,
            key,
            auction_house,
            slot,
            write_version,
        )
        .await;
        let row = client
            .query_one(
                "SELECT * FROM auction_house WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, i32>("bump"),
            auction_house.bump as i32,
            "it_upserts_auction_house: bump"
        );
    }

    async fn it_upserts_listing_receipt(
        client: &Client,
        key: &[u8],
        receipt: &ListingReceipt,
        slot: u64,
        write_version: u64,
    ) {
        queries::mpl_auction_house::listing_receipt::upsert(
            client,
            key,
            receipt,
            slot,
            write_version,
        )
        .await;
        let row = client
            .query_one(
                "SELECT * FROM listing_receipt WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, String>("auction_house"),
            receipt.auction_house.to_string(),
            "it_upserts_listing_receipt: auction_house"
        );
        assert_eq!(
            row.get::<&str, Option<i64>>("canceled_at_on_chain"),
            receipt.canceled_at,
            "it_upserts_listing_receipt: canceled_at_on_chain"
        );
    }

    async fn it_upserts_bid_receipt(
        client: &Client,
        key: &[u8],
        receipt: &BidReceipt,
        slot: u64,
        write_version: u64,
    ) {
        queries::mpl_auction_house::bid_receipt::upsert(client, key, receipt, slot, write_version)
            .await;
        let row = client
            .query_one(
                "SELECT * FROM bid_receipt WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, String>("auction_house"),
            receipt.auction_house.to_string(),
            "it_upserts_bid_receipt: auction_house"
        );
        assert_eq!(
            row.get::<&str, Option<i64>>("canceled_at_on_chain"),
            receipt.canceled_at,
            "it_upserts_bid_receipt: canceled_at_on_chain"
        );
    }

    async fn it_upserts_purchase_receipt(
        client: &Client,
        key: &[u8],
        receipt: &PurchaseReceipt,
        slot: u64,
        write_version: u64,
    ) {
        queries::mpl_auction_house::purchase_receipt::upsert(
            client,
            key,
            receipt,
            slot,
            write_version,
        )
        .await;
        let row = client
            .query_one(
                "SELECT * FROM purchase_receipt WHERE id = $1",
                &[&bs58::encode(key).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, String>("auction_house"),
            receipt.auction_house.to_string(),
            "it_upserts_purchase_receipt: auction_house"
        );
        assert_eq!(
            row.get::<&str, i64>("created_at_on_chain"),
            receipt.created_at,
            "it_upserts_purchase_receipt: created_at_on_chain"
        );
    }

    // =============================
    // Transactions
    // =============================

    async fn it_upserts_transaction(
        client: &Client,
        signature: &Signature,
        accounts: &Vec<Pubkey>,
        data: &[u8],
        slot: u64,
        table: &str,
    ) {
        if table == "create_promo" {
            queries::bpl_token_metadata::create_promo::upsert(
                client, signature, accounts, data, slot,
            )
            .await;
        } else if table == "mint_promo_token" {
            queries::bpl_token_metadata::mint_promo_token::upsert(
                client, signature, accounts, data, slot,
            )
            .await;
        } else if table == "delegate_promo_token" {
            queries::bpl_token_metadata::delegate_promo_token::upsert(
                client, signature, accounts, data, slot,
            )
            .await;
        } else if table == "burn_delegated_promo_token" {
            queries::bpl_token_metadata::delegate_promo_token::upsert(
                client, signature, accounts, data, slot,
            )
            .await;
        }

        let row = client
            .query_one(
                &format!("SELECT * FROM {table} WHERE signature = $1"),
                &[&bs58::encode(signature).into_string()],
            )
            .await
            .unwrap();
        assert_eq!(
            row.get::<&str, String>("payer"),
            accounts[0].to_string(),
            "it_upsert_{table}: payer failed"
        );
    }

    #[tokio::test]
    async fn it_runs_account_tests_success() {
        dotenv::dotenv().ok();
        tracing_subscriber::fmt().init();

        let pg_config = DatabaseURL::Localnet.url().parse::<Config>().unwrap();
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pg_pool = Pool::builder(mgr).max_size(10).build().unwrap();

        let mint_pubkey = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let supply = 4;
        let mut mint = Mint {
            mint_authority: COption::Some(mint_authority),
            freeze_authority: COption::Some(mint_authority),
            supply,
            decimals: 0,
            is_initialized: true,
        };

        let mut client = pg_pool.get().await.unwrap();
        reset(&mut client).await.unwrap();

        // insert a create_promo transaction
        let data: &[u8] = &[0, 0, 0];
        let accounts: Vec<Pubkey> = (0..10).map(|_| Pubkey::new_unique()).collect();

        for table in vec!["create_promo", "mint_promo_token", "delegate_promo_token"] {
            it_upserts_transaction(&client, &Signature::default(), &accounts, data, 42, table)
                .await;
        }

        // update a mint, null out an optional value
        mint.supply = 2;
        mint.mint_authority = COption::None;
        it_upserts_mint(&client, mint_pubkey.as_ref(), &mint, 42, 2).await;

        let owner = Pubkey::new_unique();
        let token_pubkey = get_associated_token_address(&owner, &mint_pubkey);
        let mut token_account = Account {
            mint: mint_pubkey,
            amount: 2,
            owner,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::Some(1_000_000),
            delegated_amount: 0,
            close_authority: COption::None,
        };
        // insert a token account
        it_upserts_token_account(&client, token_pubkey.as_ref(), &token_account, 42, 1).await;

        // update a token account, null out an optional value
        token_account.amount = 1;
        token_account.close_authority = COption::None;
        it_upserts_token_account(&client, token_pubkey.as_ref(), &token_account, 43, 1).await;

        // insert a metadata account
        let creators = (0..5)
            .map(|_| Creator {
                address: Pubkey::new_unique(),
                verified: false,
                share: 20,
            })
            .collect::<Vec<Creator>>();

        let (metadata_pubkey, _) = find_metadata_account(&mint_pubkey);
        let mut metadata = Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: mint_pubkey,
            data: Data {
                name: "Name".to_string(),
                symbol: "SYMBOL".to_string(),
                uri: "https://uri.tbd".to_string(),
                seller_fee_basis_points: 420,
                creators: Some(creators),
            },
            collection: Some(Collection {
                key: Pubkey::new_unique(),
                verified: false,
            }),
            primary_sale_happened: false,
            is_mutable: true,
            edition_nonce: Some(1),
            token_standard: Some(TokenStandard::FungibleAsset),
            uses: Some(Uses {
                use_method: UseMethod::Single,
                remaining: 4,
                total: 4,
            }),
        };
        it_upserts_metadata(&client, metadata_pubkey.as_ref(), &metadata, 42, 1).await;

        // update a metadata account, including change in number of creators
        metadata.data.creators = metadata.data.creators.map(|v| v.split_at(2).0.to_vec());
        metadata.data.seller_fee_basis_points = 888;
        it_upserts_metadata(&client, metadata_pubkey.as_ref(), &metadata, 42, 2).await;

        // insert edition_metadata
        let key = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let promo = Promo {
            owner,
            mint: mint_pubkey,
            metadata: metadata_pubkey,
            mint_count: 0,
            burn_count: 0,
            max_mint: Some(88),
            max_burn: Some(42),
        };

        it_upserts_promo(&client, key.as_ref(), &promo, 42, 1).await;

        // insert an auction_house
        let ah_authority = Pubkey::new_unique();
        let (ah_pubkey, _) = find_auction_house_address(&ah_authority, &mint_pubkey);
        let auction_house = AuctionHouse {
            auction_house_fee_account: Pubkey::new_unique(),
            auction_house_treasury: Pubkey::new_unique(),
            treasury_withdrawal_destination: Pubkey::new_unique(),
            fee_withdrawal_destination: Pubkey::new_unique(),
            treasury_mint: spl_token::native_mint::ID,
            authority: ah_authority,
            creator: ah_authority,
            bump: 1,
            treasury_bump: 2,
            fee_payer_bump: 3,
            seller_fee_basis_points: 4,
            requires_sign_off: false,
            can_change_sale_price: false,
        };
        it_upserts_auction_house(&client, ah_pubkey.as_ref(), &auction_house, 42, 1).await;

        // insert listing_receipt
        let price = 1_000_000_000;
        let token_size = 42;
        let (seller_trade_state, trade_state_bump) = find_trade_state_address(
            &owner,
            &ah_pubkey,
            &token_pubkey,
            &spl_token::native_mint::ID,
            &mint_pubkey,
            price,
            token_size,
        );
        let (listing_receipt_pubkey, bump) = find_listing_receipt_address(&seller_trade_state);

        let mut listing_receipt = ListingReceipt {
            trade_state: seller_trade_state,
            bookkeeper: owner,
            auction_house: ah_pubkey,
            seller: owner,
            metadata: metadata_pubkey,
            purchase_receipt: None,
            price,
            token_size,
            bump,
            trade_state_bump,
            created_at: get_now(),
            canceled_at: None,
        };
        it_upserts_listing_receipt(
            &client,
            listing_receipt_pubkey.as_ref(),
            &listing_receipt,
            42,
            1,
        )
        .await;

        // update listing_receipt
        listing_receipt.canceled_at = Some(get_now());
        it_upserts_listing_receipt(
            &client,
            listing_receipt_pubkey.as_ref(),
            &listing_receipt,
            43,
            1,
        )
        .await;

        // insert bid_receipt
        let buyer = Pubkey::new_unique();
        let (buyer_trade_state, trade_state_bump) = find_trade_state_address(
            &buyer,
            &ah_pubkey,
            &token_pubkey,
            &spl_token::native_mint::ID,
            &mint_pubkey,
            price,
            token_size,
        );
        let (bid_receipt_pubkey, bump) = find_bid_receipt_address(&buyer_trade_state);

        let mut bid_receipt = BidReceipt {
            trade_state: buyer_trade_state,
            bookkeeper: owner,
            auction_house: ah_pubkey,
            buyer,
            metadata: metadata_pubkey,
            token_account: Some(token_pubkey),
            purchase_receipt: None,
            price,
            token_size,
            bump,
            trade_state_bump,
            created_at: get_now(),
            canceled_at: None,
        };
        it_upserts_bid_receipt(&client, bid_receipt_pubkey.as_ref(), &bid_receipt, 42, 1).await;

        // update bid_receipt
        bid_receipt.canceled_at = Some(get_now());
        it_upserts_bid_receipt(&client, bid_receipt_pubkey.as_ref(), &bid_receipt, 43, 1).await;

        // insert purchase_receipt
        let (purchase_receipt_pubkey, bump) =
            find_purchase_receipt_address(&seller_trade_state, &buyer_trade_state);

        let purchase_receipt = PurchaseReceipt {
            bookkeeper: owner,
            buyer,
            seller: owner,
            auction_house: ah_pubkey,
            metadata: metadata_pubkey,
            token_size,
            price,
            bump,
            created_at: get_now(),
        };
        it_upserts_purchase_receipt(
            &client,
            purchase_receipt_pubkey.as_ref(),
            &purchase_receipt,
            42,
            1,
        )
        .await;

        listing_receipt.canceled_at = None;
        listing_receipt.purchase_receipt = Some(purchase_receipt_pubkey);
        it_upserts_listing_receipt(
            &client,
            listing_receipt_pubkey.as_ref(),
            &listing_receipt,
            44,
            1,
        )
        .await;

        bid_receipt.canceled_at = None;
        bid_receipt.purchase_receipt = Some(purchase_receipt_pubkey);
        it_upserts_bid_receipt(&client, bid_receipt_pubkey.as_ref(), &bid_receipt, 43, 2).await;
    }
}

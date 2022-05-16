pub use bpl_token_metadata::{state::Promo, ID};
use serde_json::json;
use solana_geyser_plugin_interface::geyser_plugin_interface::Result;
use std::{path::PathBuf, str::FromStr};

// TODO: parse offer terms
pub(crate) async fn process(key: &[u8], account: Promo) -> Result<()> {
    let promo = json!({
        "owner": account.owner.to_string(),
        "mint": account.mint.to_string(),
        "metadata": account.metadata.to_string(),
        "maxMint": account.max_mint,
        "maxRedeem": account.max_burn,
        "expiry": account.expiry,
    });

    let file_path = PathBuf::from_str(super::OUT_DIR)
        .unwrap()
        .join(bs58::encode(key).into_string())
        .with_extension("json");

    let s = serde_json::to_string(&promo).unwrap();

    smol::fs::write(file_path, s).await?;
    Ok(())
}

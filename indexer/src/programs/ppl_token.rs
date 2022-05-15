use crate::accounts;
use anchor_lang::AccountDeserialize;
pub use bpl_token_metadata::{state::Promo, ID};
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPluginError, ReplicaAccountInfo, Result,
};

async fn process_promo<'a>(update: &mut ReplicaAccountInfo<'a>) -> Result<()> {
    let promo: Promo = Promo::try_deserialize(&mut update.data).map_err(|_| {
        GeyserPluginError::AccountsUpdateError {
            msg: "error deserializing promo account".to_string(),
        }
    })?;
    accounts::promo::process(update.pubkey, promo).await
}

pub(crate) async fn process<'a>(update: &mut ReplicaAccountInfo<'a>) -> Result<()> {
    match update.data.len() {
        Promo::LEN => process_promo(update).await,
        _ => Ok(()),
    }
}

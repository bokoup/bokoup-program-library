use crate::{accounts, accounts_selector::AccountsSelector, programs};
use log::*;
use solana_geyser_plugin_interface::geyser_plugin_interface::{
    GeyserPlugin, ReplicaAccountInfoVersions, ReplicaBlockInfoVersions,
    ReplicaTransactionInfoVersions, Result, SlotStatus,
};
use std::{
    fs::{create_dir_all, File},
    io::Read,
    path::PathBuf,
    str::FromStr,
};

#[derive(Default)]
pub struct Plugin {
    accounts_selector: Option<AccountsSelector>,
}

impl std::fmt::Debug for Plugin {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Plugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_accounts_selector_from_config(config: &serde_json::Value) -> AccountsSelector {
        let accounts_selector = &config["accounts_selector"];

        if accounts_selector.is_null() {
            AccountsSelector::default()
        } else {
            let accounts = &accounts_selector["accounts"];
            let accounts: Vec<String> = if accounts.is_array() {
                accounts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            let owners = &accounts_selector["owners"];
            let owners: Vec<String> = if owners.is_array() {
                owners
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            AccountsSelector::new(&accounts, &owners)
        }
    }
}

impl GeyserPlugin for Plugin {
    fn name(&self) -> &'static str {
        "Plugin"
    }

    fn on_load(&mut self, config_file: &str) -> Result<()> {
        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );
        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let result: serde_json::Value = serde_json::from_str(&contents).unwrap();
        self.accounts_selector = Some(Self::create_accounts_selector_from_config(&result));

        if !PathBuf::from_str(accounts::OUT_DIR).unwrap().exists() {
            create_dir_all(accounts::OUT_DIR).unwrap()
        };

        Ok(())
    }

    fn on_unload(&mut self) {
        info!("Unloading plugin {:?}", self.name())
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        match account {
            ReplicaAccountInfoVersions::V0_0_1(account) => {
                if let Some(accounts_selector) = &self.accounts_selector {
                    if !accounts_selector.is_account_selected(account.pubkey, account.owner) {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }

                info!(
                    "Updating account {:?} with owner {:?} at slot {:?} using account selector {:?}",
                    bs58::encode(account.pubkey).into_string(),
                    bs58::encode(account.owner).into_string(),
                    slot,
                    self.accounts_selector.as_ref().unwrap()
                );

                match account {
                    update if account.owner == programs::ppl_token::ID.as_ref() => {
                        let mut update = update.clone();
                        smol::block_on(async { programs::ppl_token::process(&mut update).await })?;
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> Result<()> {
        info!("slot: {}, parent: {:?}, status: {:?}", slot, parent, status);

        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> Result<()> {
        info!("end of startup");

        Ok(())
    }

    fn notify_transaction(
        &mut self,
        transaction_info: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> Result<()> {
        info!("slot: {}, transaction_info", slot);
        Ok(())
    }

    fn notify_block_metadata(&mut self, block_info: ReplicaBlockInfoVersions) -> Result<()> {
        info!("block metadata");

        Ok(())
    }

    /// Check if the plugin is interested in account data
    /// Default is true -- if the plugin is not interested in
    /// account data, please return false.
    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    /// Check if the plugin is interested in transaction data
    fn transaction_notifications_enabled(&self) -> bool {
        true
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns the Plugin pointer as trait GeyserPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = Plugin::new();
    let plugin: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}

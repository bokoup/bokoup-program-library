use serde::{Deserialize, Serialize};

pub mod get_app_id;
pub mod get_burn_delegated_promo_tx;
pub mod get_create_promo_group_tx;
pub mod get_create_promo_tx;
pub mod get_delegate_promo_tx;
pub mod get_mint_promo_tx;

#[derive(Deserialize, Debug)]
pub struct Params {
    pub mint_string: String,
    pub message: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PromoGroupParams {
    pub group_seed: String,
    pub members: Vec<String>,
    pub lamports: u64,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BurnDelegatedParams {
    pub token_account_string: String,
    pub message: String,
    pub memo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CreatePromoParams {
    pub payer: String,
    pub group_seed: String,
    pub memo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PayResponse {
    pub transaction: String,
    pub message: String,
}

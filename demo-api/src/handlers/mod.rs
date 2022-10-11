use serde::Deserialize;

pub mod get_app_id;
pub mod get_burn_delegated_promo_tx;
pub mod get_delegate_promo_tx;
pub mod get_mint_promo_tx;

#[derive(Deserialize, Debug)]
pub struct Params {
    pub mint_string: String,
    pub message: String,
    pub memo: String,
}

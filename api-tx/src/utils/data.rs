use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use serde_json::Value;

use crate::error::AppError;

pub const MINT_QUERY: &str = r#"
    query MintQuery {
        mint(limit: 1) {
            id
            promoObject {
                groupObject {
                id
                members
                }
            }
        }
    }
    "#;

pub const PROMO_GROUP_QUERY: &str = r#"
query MintQuery($mint: String!) {
    mintByPk(id: $mint) {
      promoObject {
        groupObject {
          id
          seed
          members
        }
      }
    }
  }
  "#;

/// Looks up mint in data api, checks to make sure payer
/// is included in group and returns group address. If mint it not found
/// or payer is not in group returns and error so that customer requesting
/// url gets error from server before trying to submit transaction to the network.
pub fn get_group(payer: &Pubkey, result: &Value) -> Result<Pubkey, AppError> {
    let mint_obj = result
        .as_object()
        .unwrap()
        .get("data")
        .unwrap()
        .as_object()
        .unwrap()
        .get("mintByPk");

    if let Some(mint) = mint_obj {
        let group_obj = mint
            .get("promoObject")
            .unwrap()
            .as_object()
            .unwrap()
            .get("groupObject")
            .unwrap()
            .as_object()
            .unwrap();
        let group_str = group_obj.get("id").unwrap().as_str().unwrap();

        let group_members: Vec<Pubkey> = group_obj
            .get("members")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|s| Pubkey::from_str(s.as_str().unwrap()).unwrap())
            .collect();
        if group_members.contains(payer) {
            Ok(Pubkey::from_str(group_str).unwrap())
        } else {
            Err(AppError::PayerNotInMembers)
        }
    } else {
        Err(AppError::DataQueryError)
    }
}

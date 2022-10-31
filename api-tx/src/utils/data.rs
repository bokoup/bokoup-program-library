use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use serde_json::Value;

use crate::error::AppError;

pub const FIRST_MINT_QUERY: &str = r#"
    query FirstMintQuery {
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

pub const FIRST_TOKEN_ACCOUNT_QUERY: &str = r#"
    query FirstTokenAccount {
        tokenAccount(limit: 1) {
        id
        }
    }
    "#;

pub const MINT_QUERY: &str = r#"
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

pub const TOKEN_ACCOUNT_QUERY: &str = r#"
query TokenAccountQuery($id: String!) {
    tokenAccountByPk(id: $id) {
      id
      owner
      mint
      delegate
      delegatedAmount
      mintObject {
        promoObject {
          groupObject {
            id
            seed
            members
          }
        }
      }
    }
  }  
  "#;

pub fn get_mint_object_from_promo_group_query(result: &Value) -> Option<&Value> {
    result
        .as_object()
        .unwrap()
        .get("data")
        .unwrap()
        .as_object()
        .unwrap()
        .get("mintByPk")
}

pub fn get_token_account_object_from_token_account_query(result: &Value) -> Option<&Value> {
    result
        .as_object()
        .unwrap()
        .get("data")
        .unwrap()
        .as_object()
        .unwrap()
        .get("tokenAccountByPk")
}

pub fn get_mint_owner_group_from_token_account_query(
    payer: &Pubkey,
    result: &Value,
) -> Result<(Pubkey, Pubkey, Pubkey), AppError> {
    let token_account_obj = get_token_account_object_from_token_account_query(result);
    if let Some(token_account) = token_account_obj {
        let mint = token_account
            .get("mint")
            .unwrap()
            .as_str()
            .map(Pubkey::from_str)
            .unwrap()?;

        let owner = token_account
            .get("owner")
            .unwrap()
            .as_str()
            .map(Pubkey::from_str)
            .unwrap()?;

        let mint_obj = token_account.get("mintObject");

        let group = get_group_from_mint_object(payer, mint_obj)?;

        Ok((mint, owner, group))
    } else {
        Err(AppError::DataQueryError)
    }
}

/// Looks up mint in data api, checks to make sure payer
/// is included in group and returns group address. If mint it not found
/// or payer is not in group returns and error so that customer requesting
/// url gets error from server before trying to submit transaction to the network.
pub fn get_group_from_promo_group_query(
    payer: &Pubkey,
    result: &Value,
) -> Result<Pubkey, AppError> {
    let mint_obj = get_mint_object_from_promo_group_query(result);
    get_group_from_mint_object(payer, mint_obj)
}

pub fn get_group_from_mint_object(
    payer: &Pubkey,
    mint_obj: Option<&Value>,
) -> Result<Pubkey, AppError> {
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

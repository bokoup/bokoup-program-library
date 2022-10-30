use crate::{
    error::AppError,
    utils::{
        bundlr::{upload_image, upload_metadata_json},
        solana::create_create_promo_instruction,
    },
    State,
};
use anchor_lang::prelude::Pubkey;
use axum::{extract::Multipart, Extension, Json};
use serde_json::{Map, Value};
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use std::{str::FromStr, sync::Arc};

pub async fn handler(
    mut multipart: Multipart,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<Value>, AppError> {
    // Parse data - two parts - json data and image.
    let mut metadata_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "metadata" {
            let json_string = field.text().await.map_err(|_| {
                AppError::CreatePromoRequestError("metadata value not valid".to_string())
            })?;
            Ok(serde_json::from_str::<Value>(&json_string)?)
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::CreatePromoRequestError(
            "request had no parts".to_string(),
        ))
    }?;

    let image_data = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "image" {
            let content_type = field.content_type().map(ToString::to_string).ok_or(
                AppError::CreatePromoRequestError("failed to read image content type".to_string()),
            )?;
            let image_bytes = field.bytes().await.map_err(|_| {
                AppError::CreatePromoRequestError("failed to read image bytes".to_string())
            })?;
            Ok((image_bytes.to_vec(), content_type))
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::CreatePromoRequestError(
            "request only had one part".to_string(),
        ))
    }?;

    let group_seed_string = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "groupSeed" {
            let memo_string = field.text().await.map_err(|_| {
                AppError::CreatePromoRequestError("groupSeed value not valid".to_string())
            })?;
            Ok(memo_string)
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        Err(AppError::CreatePromoRequestError(
            "groupSeed field did not exist".to_string(),
        ))
    }?;

    let memo = if let Some(field) = multipart.next_field().await.unwrap() {
        if field.name().expect("name field should exist") == "memo" {
            let memo_string = field.text().await.map_err(|_| {
                AppError::CreatePromoRequestError("memo value not valid".to_string())
            })?;
            Some(memo_string)
        } else {
            return Err(AppError::CreatePromoRequestError(
                "invalid field name".to_string(),
            ));
        }
    } else {
        None
    };

    let metadata_data_obj =
        metadata_data
            .as_object_mut()
            .ok_or(AppError::CreatePromoRequestError(
                "metadata data part should be an object".to_string(),
            ))?;

    // Upload image to Arweave.
    let (image_url, content_type, state) = upload_image(image_data, state).await?;

    // Upload metadata json to Arweave.
    let (uri, state) =
        upload_metadata_json(metadata_data_obj, image_url, content_type, state).await?;

    // Parse promo args.
    let (name, symbol, max_mint, max_burn) = get_promo_args(metadata_data_obj)?;
    let mint_keypair = Keypair::new();

    let group_seed = Pubkey::from_str(&group_seed_string)?;
    // Create promo instruction.
    let ix = create_create_promo_instruction(
        state.promo_owner.pubkey(),
        group_seed,
        mint_keypair.pubkey(),
        state.platform,
        name,
        symbol,
        uri,
        max_mint,
        max_burn,
        true,
        memo,
    )?;

    let mut tx = Transaction::new_with_payer(&[ix], Some(&state.promo_owner.pubkey()));
    let latest_blockhash = state.solana.get_latest_blockhash().await?;
    tx.sign(&[&state.promo_owner, &mint_keypair], latest_blockhash);

    let serialized = bincode::serialize(&tx)?;
    let tx_str = base64::encode(serialized);
    let result = state.solana.post_transaction_test(&tx_str).await?;

    tracing::debug!(result = format!("{:?}", result));

    Ok(Json(result))
}

pub fn get_promo_args(
    metadata_data_obj: &mut Map<String, Value>,
) -> Result<(String, String, Option<u32>, Option<u32>), AppError> {
    let name = metadata_data_obj["name"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "name field should exist".to_string(),
        ))?
        .to_string();

    let symbol = metadata_data_obj["symbol"]
        .as_str()
        .ok_or(AppError::CreatePromoRequestError(
            "symbol field should exist".to_string(),
        ))?
        .to_string();

    // Return max_mint and max_burn if attributes exists in json data.
    let (max_mint, max_burn) = if let Some(value) = metadata_data_obj.get("attributes") {
        if let Some(attributes) = value.as_array() {
            let max_mint: Option<u32> = attributes
                .iter()
                .filter_map(|a| {
                    let attribute = a.as_object()?;
                    if let Some(trait_type) = attribute.get("trait_type") {
                        if trait_type == "maxMint" {
                            attribute.get("value").map(|v| v.as_u64()).unwrap_or(None)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<u64>>()
                .first()
                .map(|v| v.clone() as u32);

            let max_burn: Option<u32> = attributes
                .iter()
                .filter_map(|a| {
                    let attribute = a.as_object()?;
                    if let Some(trait_type) = attribute.get("trait_type") {
                        if trait_type == "maxBurn" {
                            attribute.get("value").map(|v| v.as_u64()).unwrap_or(None)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<u64>>()
                .first()
                .map(|v| v.clone() as u32);

            (max_mint, max_burn)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    Ok((name, symbol, max_mint, max_burn))
}

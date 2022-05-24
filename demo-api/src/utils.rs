use crate::error::AppError;
use anchor_lang::{
    prelude::Pubkey,
    InstructionData, ToAccountMetas,
    {
        solana_program::{instruction::Instruction, sysvar},
        system_program,
    },
};
use bpl_token_metadata::{
    accounts::MintPromoToken as mint_promo_token_accounts,
    instruction::MintPromoToken as mint_promo_token_instruction,
    utils::{
        find_admin_address, find_associated_token_address, find_authority_address,
        find_promo_address,
    },
};
use futures::join;

pub async fn create_transfer_promo_instruction(
    wallet: Pubkey,
    mint: Pubkey,
) -> Result<Instruction, AppError> {
    let (
        (authority, _auth_bump),
        (promo, _promo_bump),
        (admin_settings, _admin_bump),
        token_account,
    ) = join!(
        find_authority_address(),
        find_promo_address(&mint),
        find_admin_address(),
        find_associated_token_address(&wallet, &mint)
    );

    let accounts = mint_promo_token_accounts {
        payer: wallet,
        mint: mint,
        authority,
        promo,
        admin_settings,
        token_account,
        token_program: anchor_spl::token::ID,
        associated_token_program: anchor_spl::associated_token::ID,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
    }
    .to_account_metas(Some(true));

    let data = mint_promo_token_instruction {}.data();

    Ok(Instruction {
        program_id: bpl_token_metadata::id(),
        accounts,
        data,
    })
}

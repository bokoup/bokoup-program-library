use crate::{CreateMetaData, CreateNonFungible, TransferSol};
use anchor_lang::prelude::*;
use mpl_token_metadata::state::DataV2;

pub const ADMIN_PREFIX: &str = "admin";
pub const AUTHORITY_PREFIX: &str = "authority";
pub const PROMO_PREFIX: &str = "promo";

pub fn transfer_sol<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, TransferSol<'info>>,
    lamports: u64,
) -> Result<()> {
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.payer.key(),
        &ctx.accounts.to.key(),
        lamports,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.to.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_metadata_accounts_v2<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateMetaData<'info>>,
    update_authority_is_signer: bool,
    is_mutable: bool,
    data: DataV2,
) -> Result<()> {
    let ix = mpl_token_metadata::instruction::create_metadata_accounts_v2(
        mpl_token_metadata::ID.clone(),
        ctx.accounts.metadata_account.to_account_info().key(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.mint_authority.key(),
        ctx.accounts.payer.key(),
        ctx.accounts.metadata_authority.key(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        update_authority_is_signer,
        is_mutable,
        data.collection,
        data.uses,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.metadata_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn create_master_edition_v3<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateNonFungible<'info>>,
    max_supply: Option<u64>,
) -> Result<()> {
    let ix = mpl_token_metadata::instruction::create_master_edition_v3(
        mpl_token_metadata::ID.clone(),
        ctx.accounts.edition_account.key.clone(),
        ctx.accounts.mint.to_account_info().key(),
        ctx.accounts.authority.key.clone(),
        ctx.accounts.authority.key.clone(),
        ctx.accounts.metadata_account.key(),
        ctx.accounts.payer.key.clone(),
        max_supply,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            ctx.accounts.edition_account.to_account_info(),
            ctx.accounts.metadata_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

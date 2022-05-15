use crate::{
    state::DataV2,
    utils::{create_master_edition_v3, create_metadata_accounts_v2},
    CreateMetaData, CreateNonFungible,
};
use anchor_lang::prelude::*;

impl<'info> CreateNonFungible<'info> {
    pub fn process(
        &mut self,
        data: DataV2,
        is_mutable: bool,
        max_supply: Option<u64>,
        mint_authority_seeds: [&[u8]; 2],
    ) -> Result<()> {
        msg!("Create non-fungible");

        let mint_to_ctx = anchor_spl::token::MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_account.to_account_info(),
            authority: self.authority.to_account_info(),
        };

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_to_ctx,
                &[&mint_authority_seeds],
            ),
            1,
        )?;

        create_metadata_accounts_v2(
            CpiContext::new_with_signer(
                self.metadata_program.to_account_info(),
                CreateMetaData::from(self.clone()),
                &[&mint_authority_seeds],
            ),
            false,
            is_mutable,
            data.into(),
        )?;

        create_master_edition_v3(
            CpiContext::new_with_signer(
                self.metadata_program.to_account_info(),
                self.clone(),
                &[&mint_authority_seeds],
            ),
            max_supply,
        )?;
        Ok(())
    }
}

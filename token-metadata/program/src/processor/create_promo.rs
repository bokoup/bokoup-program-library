use crate::{
    state::{DataV2, Promo},
    utils::{create_metadata_accounts_v2, transfer_sol},
    CreateMetaData, CreatePromo, TransferSol,
};
use anchor_lang::prelude::*;

impl<'info> CreatePromo<'info> {
    pub fn process(
        &mut self,
        promo_data: Promo,
        metadata_data: DataV2,
        is_mutable: bool,
        authority_seeds: [&[u8]; 2],
    ) -> Result<()> {
        msg!("Create promo");

        if self.admin_settings.create_promo_lamports > 0 {
            transfer_sol(
                CpiContext::new(
                    self.system_program.to_account_info(),
                    TransferSol {
                        payer: self.payer.to_account_info(),
                        to: self.platform.to_account_info(),
                        system_program: self.system_program.clone(),
                    },
                ),
                self.admin_settings.create_promo_lamports,
            )?;
        }

        create_metadata_accounts_v2(
            CpiContext::new_with_signer(
                self.metadata_program.to_account_info(),
                CreateMetaData::from(self.clone()),
                &[&authority_seeds],
            ),
            false,
            is_mutable,
            metadata_data.into(),
        )?;

        **self.promo = promo_data;
        Ok(())
    }
}

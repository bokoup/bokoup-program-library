use crate::{
    state::{DataV2, Promo},
    utils::{create_memo, create_metadata_accounts_v2},
    CreateMetaData, CreatePromo,
};
use anchor_lang::prelude::*;

impl<'info> CreatePromo<'info> {
    pub fn process(
        &mut self,
        promo_data: Promo,
        metadata_data: DataV2,
        is_mutable: bool,
        authority_seeds: [&[u8]; 2],
        memo: Option<String>,
    ) -> Result<()> {
        msg!("Create promo");

        // Error if not enough lamports
        if self.group.to_account_info().lamports.borrow().clone()
            < self.admin_settings.create_promo_lamports
        {
            return Err(ProgramError::InsufficientFunds.into());
        }

        if self.admin_settings.create_promo_lamports > 0 {
            let group = self.group.to_account_info();
            let platform = self.platform.to_account_info();
            let amount = self.admin_settings.create_promo_lamports;

            **group.try_borrow_mut_lamports()? = group.lamports().checked_sub(amount).unwrap();
            **platform.try_borrow_mut_lamports()? =
                platform.lamports().checked_add(amount).unwrap();
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

        if let Some(memo) = memo {
            let account_infos = vec![self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }

        *self.promo = promo_data;
        Ok(())
    }
}

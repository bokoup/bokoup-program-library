use crate::{
    state::PromoGroup,
    utils::{create_memo, transfer_sol},
    CreatePromoGroup, TransferSol,
};
use anchor_lang::prelude::*;

impl<'info> CreatePromoGroup<'info> {
    pub fn process(&mut self, data: PromoGroup, lamports: u64, memo: Option<String>) -> Result<()> {
        msg!("Create group");

        *self.group = data;

        transfer_sol(
            CpiContext::new(
                self.system_program.to_account_info(),
                TransferSol {
                    payer: self.payer.to_account_info(),
                    to: self.group.to_account_info(),
                },
            ),
            lamports,
        )?;

        if let Some(memo) = memo {
            let account_infos = vec![self.payer.to_account_info()];
            create_memo(memo, account_infos)?;
        }

        Ok(())
    }
}

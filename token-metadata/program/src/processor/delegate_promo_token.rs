use crate::DelegatePromoToken;
use anchor_lang::prelude::*;

impl<'info> DelegatePromoToken<'info> {
    pub fn process(&mut self) -> Result<()> {
        msg!("Delegate promo token");

        let delegate_ctx = anchor_spl::token::Approve {
            to: self.token_account.to_account_info(),
            delegate: self.authority.to_account_info(),
            authority: self.token_owner.to_account_info(),
        };

        anchor_spl::token::approve(
            CpiContext::new(self.token_program.to_account_info(), delegate_ctx),
            1,
        )?;

        Ok(())
    }
}

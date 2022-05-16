use crate::{error::ProgramError, DelegatePromoToken};
use anchor_lang::prelude::*;

impl<'info> DelegatePromoToken<'info> {
    pub fn process(&mut self) -> Result<()> {
        msg!("Delegate promo token");

        // if let Some(max_redeem) = self.promo.max_redeem {
        //     if self.mint.supply >= max_mint as u64 {
        //         return Err(ProgramError::MaxMintExceeded.into());
        //     }
        // }

        if let Some(expiry) = self.promo.expiry {
            let clock = Clock::get()?;
            if clock.unix_timestamp >= expiry {
                return Err(ProgramError::ExpiryExceeded.into());
            }
        }

        let delegate_ctx = anchor_spl::token::Approve {
            to: self.token_account.to_account_info(),
            delegate: self.authority.to_account_info(),
            authority: self.payer.to_account_info(),
        };

        anchor_spl::token::approve(
            CpiContext::new(self.token_program.to_account_info(), delegate_ctx),
            1,
        )?;

        Ok(())
    }
}

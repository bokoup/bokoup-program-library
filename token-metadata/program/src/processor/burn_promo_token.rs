use crate::{error::ProgramError, BurnPromoToken};
use anchor_lang::prelude::*;

impl<'info> BurnPromoToken<'info> {
    pub fn process(&mut self, authority_seeds: [&[u8]; 2]) -> Result<()> {
        msg!("Burn promo token");

        if let Some(max_burn) = self.promo.max_burn {
            if self.promo.burns >= max_burn {
                return Err(ProgramError::MaxMintExceeded.into());
            }
        }

        if let Some(expiry) = self.promo.expiry {
            let clock = Clock::get()?;
            if clock.unix_timestamp >= expiry {
                return Err(ProgramError::ExpiryExceeded.into());
            }
        }

        let burn_ctx = anchor_spl::token::Burn {
            mint: self.mint.to_account_info(),
            from: self.token_account.to_account_info(),
            authority: self.authority.to_account_info(),
        };

        anchor_spl::token::burn(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                burn_ctx,
                &[&authority_seeds],
            ),
            1,
        )?;

        self.promo.burns += 1;

        Ok(())
    }
}

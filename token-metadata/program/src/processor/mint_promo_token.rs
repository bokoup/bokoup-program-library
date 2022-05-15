use crate::{error::ProgramError, MintPromoToken};
use anchor_lang::prelude::*;

impl<'info> MintPromoToken<'info> {
    pub fn process(&mut self, authority_seeds: [&[u8]; 2]) -> Result<()> {
        msg!("Issue promo token");

        if let Some(max_mint) = self.promo.max_mint {
            if self.mint.supply >= max_mint as u64 {
                return Err(ProgramError::MaxMintExceeded.into());
            }
        }

        if let Some(expiry) = self.promo.expiry {
            let clock = Clock::get()?;
            if clock.unix_timestamp >= expiry {
                return Err(ProgramError::ExpiryExceeded.into());
            }
        }

        let mint_to_ctx = anchor_spl::token::MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_account.to_account_info(),
            authority: self.authority.to_account_info(),
        };

        anchor_spl::token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                mint_to_ctx,
                &[&authority_seeds],
            ),
            1,
        )?;

        Ok(())
    }
}

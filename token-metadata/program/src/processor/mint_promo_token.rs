use crate::{error::ProgramError, state::MintEvent, MintPromoToken};
use anchor_lang::prelude::*;

impl<'info> MintPromoToken<'info> {
    pub fn process(&mut self, authority_seeds: [&[u8]; 2]) -> Result<()> {
        msg!("Mint promo token");
        emit!(MintEvent {
            mint: self.mint.key().to_string(),
            token_account: self.token_account.key().to_string(),
        });

        if let Some(max_mint) = self.promo.max_mint {
            if self.promo.mints >= max_mint {
                return Err(ProgramError::MaxMintExceeded.into());
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

        self.promo.mints += 1;

        Ok(())
    }
}

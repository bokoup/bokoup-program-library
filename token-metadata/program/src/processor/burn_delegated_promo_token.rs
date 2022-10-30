use crate::utils::{create_memo, transfer_sol};
use crate::{error::ProgramError, BurnDelegatedPromoToken, TransferSol};
use anchor_lang::prelude::*;

impl<'info> BurnDelegatedPromoToken<'info> {
    pub fn process(&mut self, memo: Option<String>) -> Result<()> {
        msg!("Burn delegated promo token");

        // Check to see if burn_count is still below max_burn.
        if let Some(max_burn) = self.promo.max_burn {
            if self.promo.burn_count >= max_burn {
                return Err(ProgramError::MaxBurnExceeded.into());
            }
        }

        if self.admin_settings.burn_promo_token_lamports > 0 {
            transfer_sol(
                CpiContext::new(
                    self.system_program.to_account_info(),
                    TransferSol {
                        payer: self.payer.to_account_info(),
                        to: self.platform.to_account_info(),
                    },
                ),
                self.admin_settings.burn_promo_token_lamports,
            )?;
        }

        let burn_ctx = anchor_spl::token::Burn {
            mint: self.mint.to_account_info(),
            from: self.token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };

        anchor_spl::token::burn(
            CpiContext::new(self.token_program.to_account_info(), burn_ctx),
            1,
        )?;

        if let Some(memo) = memo {
            let account_infos = vec![self.payer.to_account_info()];
            create_memo(memo.to_string(), account_infos)?;
        }

        self.promo.burn_count += 1;

        Ok(())
    }
}

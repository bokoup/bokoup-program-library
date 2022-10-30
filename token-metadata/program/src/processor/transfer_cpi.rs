use crate::{utils::transfer_sol, TransferCpi, TransferSol};
use anchor_lang::prelude::*;

impl<'info> TransferCpi<'info> {
    pub fn process(&mut self, lamports: u64, group_seeds: [&[u8]; 2]) -> Result<()> {
        msg!("Transfer cpi");

        transfer_sol(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                TransferSol {
                    payer: self.group.to_account_info(),
                    to: self.platform.to_account_info(),
                },
                &[&group_seeds[..]],
            ),
            lamports,
        )?;

        Ok(())
    }
}

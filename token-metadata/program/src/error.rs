//! Module provide program defined errors

use anchor_lang::prelude::*;

#[error_code]
pub enum ProgramError {
    #[msg("Max mints exceeded")]
    MaxMintExceeded,
    #[msg("Max burns exceeded")]
    MaxBurnExceeded,
    #[msg("Expiry exceeded")]
    ExpiryExceeded,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Constraint not met")]
    ConstraintNotMet,
}

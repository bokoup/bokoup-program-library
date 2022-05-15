//! Module provide program defined errors

use anchor_lang::prelude::*;

#[error_code]
pub enum ProgramError {
    #[msg("Max mints exceeded.")]
    MaxMintExceeded,
    #[msg("Expiry exceeded.")]
    ExpiryExceeded,
}

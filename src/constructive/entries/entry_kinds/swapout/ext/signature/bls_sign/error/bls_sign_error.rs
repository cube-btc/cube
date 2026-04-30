use crate::constructive::entry::entry_kinds::swapout::ext::signature::sighash::error::sighash_error::SwapoutSighashError;

/// Errors associated with signing a `Swapout`.
#[derive(Debug, Clone)]
pub enum SwapoutBLSSignError {
    SighashError(SwapoutSighashError),
}

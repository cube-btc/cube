use crate::constructive::entry::entry_kinds::swapout::ext::signature::sighash::error::sighash_error::SwapoutSighashError;

/// Errors associated with verifying a BLS signature over a `Swapout`.
#[derive(Debug, Clone)]
pub enum SwapoutBLSVerifyError {
    SighashError(SwapoutSighashError),
    InvalidBLSSignatureError,
}

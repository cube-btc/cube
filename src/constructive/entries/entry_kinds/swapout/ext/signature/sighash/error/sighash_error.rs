use crate::constructive::entry::entry_kinds::swapout::ext::codec::sbe::encode::error::encode_error::SwapoutSBEEncodeError;

/// Errors associated with generating a sighash for a `Swapout`.
#[derive(Debug, Clone)]
pub enum SwapoutSighashError {
    SBEEncodeError(SwapoutSBEEncodeError),
}

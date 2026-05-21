use crate::constructive::entries::entry_kinds::call::ext::codec::sbe::encode::error::encode_error::CallSBEEncodeError;

/// Errors associated with generating a sighash for a `Call`.
#[derive(Debug, Clone)]
pub enum CallSighashError {
    SBEEncodeError(CallSBEEncodeError),
}

use crate::constructive::entries::entry_kinds::r#move::ext::codec::sbe::encode::error::encode_error::MoveSBEEncodeError;

/// Errors associated with generating a sighash for a `Move`.
#[derive(Debug, Clone)]
pub enum MoveSighashError {
    SBEEncodeError(MoveSBEEncodeError),
}

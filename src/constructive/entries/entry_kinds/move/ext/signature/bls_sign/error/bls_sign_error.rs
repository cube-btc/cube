use crate::constructive::entries::entry_kinds::r#move::ext::signature::sighash::error::sighash_error::MoveSighashError;

/// Errors associated with signing a `Move` with BLS.
#[derive(Debug, Clone)]
pub enum MoveBLSSignError {
    SighashError(MoveSighashError),
}

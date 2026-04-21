use crate::constructive::entries::entry_kinds::r#move::ext::signature::sighash::error::sighash_error::MoveSighashError;

/// Errors associated with verifying a BLS signature over a `Move`.
#[derive(Debug, Clone)]
pub enum MoveBLSVerifyError {
    SighashError(MoveSighashError),
    InvalidBLSSignatureError,
}

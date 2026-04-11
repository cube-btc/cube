use crate::constructive::entries::entry_kinds::liftup::ext::signature::sighash::error::sighash_error::LiftupSighashError;

/// Errors associated with verifying a BLS signature over a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupBLSVerifyError {
    SighashError(LiftupSighashError),
    InvalidBLSSignatureError,
}

use crate::constructive::entries::entry_kinds::liftup::ext::signature::sighash::error::sighash_error::LiftupSighashError;

/// Errors associated with signing a `Liftup` with BLS.
#[derive(Debug, Clone)]
pub enum LiftupBLSSignError {
    SighashError(LiftupSighashError),
}

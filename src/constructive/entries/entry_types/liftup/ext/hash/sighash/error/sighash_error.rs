use crate::constructive::entries::entry_types::liftup::ext::codec::sbe::encode::error::LiftupSBEEncodeError;

/// Errors associated with generating a sighash for a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupSighashError {
    SBEEncodeError(LiftupSBEEncodeError),
}
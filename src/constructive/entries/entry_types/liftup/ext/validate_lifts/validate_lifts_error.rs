use crate::constructive::txo::lift::lift::Lift;

/// Errors associated with validating the `Lift`s in a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupValidateLiftsError {
    InvalidLiftScriptpubkeyError(Lift),
    FailedToValidateLiftWithTheUTXOSetError(Lift),
}

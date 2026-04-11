use crate::constructive::txo::lift::lift::Lift;

/// Errors associated with validating the `Lift`s in a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupValidateLiftsError {
    LiftAccountKeyDoesNotMatchSelfAccountKeyError(Lift),
    LiftEngineKeyDoesNotMatchEngineKeyError(Lift),
    LiftV1ScriptpubkeyValidationError(Lift),
    LiftV2ScriptpubkeyValidationError(Lift),
    FailedToValidateLiftWithTheUTXOSetError(Lift),
}

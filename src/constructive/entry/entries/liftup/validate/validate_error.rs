use crate::constructive::entity::account::root_account::ext::validate::validate_error::RootAccountValidateError;

/// Errors associated with validating a `Liftup`.
#[derive(Debug, Clone)]
pub enum LiftupValidateError {
    RootAccountValidationError(RootAccountValidateError),
    InvalidLiftStructureError,
    InvalidLiftUTXOError,
}

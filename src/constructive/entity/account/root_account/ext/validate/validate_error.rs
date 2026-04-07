use crate::constructive::entity::account::root_account::unregistered_root_account::ext::validate::validate_error::UnregisteredRootAccountValidateError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::validate::validate_error::RegisteredButUnconfiguredRootAccountValidateError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::validate::validate_error::RegisteredAndConfiguredRootAccountValidateError;

/// Errors associated with validating a `RootAccount`.
#[derive(Debug, Clone)]
pub enum RootAccountValidateError {
    UnregisteredRootAccountValidateError(UnregisteredRootAccountValidateError),
    RegisteredButUnconfiguredRootAccountValidateError(RegisteredButUnconfiguredRootAccountValidateError),
    RegisteredAndConfiguredRootAccountValidateError(RegisteredAndConfiguredRootAccountValidateError),
}

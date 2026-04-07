/// Errors associated with validating a `RegisteredAndConfiguredRootAccount`.
#[derive(Debug, Clone)]
pub enum RegisteredAndConfiguredRootAccountValidateError {
    RegisteryIndexMismatchError,
    BLSKeyMismatchError,
    AccountIsNotRegisteredError,
}

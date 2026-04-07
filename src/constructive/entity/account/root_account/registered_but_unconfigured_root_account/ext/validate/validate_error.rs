/// Errors associated with validating a `RegisteredAndConfiguredRootAccount`.
#[derive(Debug, Clone)]
pub enum RegisteredButUnconfiguredRootAccountValidateError {
    InvalidBLSKeyError,
    BLSKeyIsConflictingError,
    RegisteryIndexMismatchError,
    BLSKeyIsAlreadyConfiguredError,
    AccountIsNotRegisteredError,
    FailedToVerifyAuthorizationSignatureError,
}

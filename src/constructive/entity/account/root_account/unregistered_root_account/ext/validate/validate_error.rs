/// Errors associated with validating a `UnregisteredRootAccount`.
#[derive(Debug, Clone)]
pub enum UnregisteredRootAccountValidateError {
    InvalidAccountKeyError,
    InvalidBLSKeyError,
    AccountIsAlreadyRegisteredError,
    BLSKeyIsConflictingWithAnAlreadyRegisteredBLSKeyError,
    AccountIsAlreadyBurriedError,
    FailedToVerifyAuthorizationSignatureError,
}

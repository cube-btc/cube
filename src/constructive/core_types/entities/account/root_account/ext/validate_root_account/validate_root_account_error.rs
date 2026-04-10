/// Errors associated with validating a `RootAccount` against the `Registery` and `Graveyard`.
#[derive(Debug, Clone)]
pub enum RootAccountValidateRootAccountError {
    // --- `UnregisteredRootAccount`
    UnregisteredValidateSchnorrAndBLSKeyError,
    UnregisteredInvalidAuthorizationSignatureError,
    UnregisteredAccountBurriedInGraveyardError,
    UnregisteredAccountRegisteredInRegisteryError,

    // --- `RegisteredButUnconfiguredRootAccount`
    RegisteredButUnconfiguredValidateBLSKeyError,
    RegisteredButUnconfiguredInvalidAuthorizationSignatureError,
    RegisteredButUnconfiguredAccountNotRegisteredInRegisteryError,
    RegisteredButUnconfiguredRegisteryIndexMismatchError,
    RegisteredButUnconfiguredBLSKeyAlreadyConfiguredInRegisteryError,

    // --- `RegisteredAndConfiguredRootAccount`
    RegisteredAndConfiguredValidateSchnorrAndBLSKeyError,
    RegisteredAndConfiguredAccountNotRegisteredInRegisteryError,
    RegisteredAndConfiguredRegisteryIndexMismatchError,
    RegisteredAndConfiguredBLSKeyNotConfiguredInRegisteryError,
    RegisteredAndConfiguredBLSKeyMismatchWithRegisteryError,
}

/// Errors associated with validating a `RootAccount` against the `Registry` and `Graveyard`.
#[derive(Debug, Clone)]
pub enum RootAccountValidateRootAccountError {
    // --- `UnregisteredRootAccount`
    UnregisteredValidateSchnorrAndBLSKeyError,
    UnregisteredInvalidAuthorizationSignatureError,
    UnregisteredAccountBuriedInGraveyardError,
    UnregisteredAccountRegisteredInRegistryError,

    // --- `RegisteredButUnconfiguredRootAccount`
    RegisteredButUnconfiguredValidateBLSKeyError,
    RegisteredButUnconfiguredInvalidAuthorizationSignatureError,
    RegisteredButUnconfiguredAccountNotRegisteredInRegistryError,
    RegisteredButUnconfiguredRegistryIndexMismatchError,
    RegisteredButUnconfiguredBLSKeyAlreadyConfiguredInRegistryError,

    // --- `RegisteredAndConfiguredRootAccount`
    RegisteredAndConfiguredValidateSchnorrAndBLSKeyError,
    RegisteredAndConfiguredAccountNotRegisteredInRegistryError,
    RegisteredAndConfiguredRegistryIndexMismatchError,
    RegisteredAndConfiguredBLSKeyNotConfiguredInRegistryError,
    RegisteredAndConfiguredBLSKeyMismatchWithRegistryError,
}

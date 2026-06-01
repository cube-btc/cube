/// Errors associated with validating an `Account` against the `Registry` and `Graveyard`.
#[derive(Debug, Clone)]
pub enum AccountValidateAccountError {
    // --- `UnregisteredAccount`
    UnregisteredValidateSchnorrKeyError,
    UnregisteredAccountBuriedInGraveyardError,
    UnregisteredAccountRegisteredInRegistryError,

    // --- `RegisteredAccount`
    RegisteredAccountNotRegisteredInRegistryError,
    RegisteredRegistryIndexMismatchError,
}

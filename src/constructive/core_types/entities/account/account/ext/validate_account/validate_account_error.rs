/// Errors associated with validating an `Account` against the `Registery` and `Graveyard`.
#[derive(Debug, Clone)]
pub enum AccountValidateAccountError {
    // --- `UnregisteredAccount`
    UnregisteredValidateSchnorrKeyError,
    UnregisteredAccountBurriedInGraveyardError,
    UnregisteredAccountRegisteredInRegisteryError,

    // --- `RegisteredAccount`
    RegisteredAccountNotRegisteredInRegisteryError,
    RegisteredRegisteryIndexMismatchError,
}

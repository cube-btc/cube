/// Account Key.
type AccountKey = [u8; 32];

/// BLS Key.
type AccountBLSKey = [u8; 48];

/// Errors associated with registering a new account.
#[derive(Debug, Clone)]
pub enum RMRegisterAccountError {
    AccountHasJustBeenEphemerallyRegistered(AccountKey),
    AccountIsAlreadyPermanentlyRegistered(AccountKey),
    BLSKeyIsConflictingWithAnAlreadyRegisteredBLSKey(AccountBLSKey),
}

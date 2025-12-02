/// Account Key.
type AccountKey = [u8; 32];

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Errors associated with updating an account's BLS key.
#[derive(Debug, Clone)]
pub enum RMReconfigAccountError {
    AccountIsNotRegistered(AccountKey),
    BLSKeyIsAlreadyPermanentlySet(AccountKey),
    BLSKeyIsAlreadyEpheremallySet(AccountKey, AccountBLSKey),
}

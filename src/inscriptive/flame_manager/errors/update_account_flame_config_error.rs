/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with updating an account's flame config.
#[derive(Debug, Clone)]
pub enum FMUpdateAccountFlameConfigError {
    AccountIsNotRegistered(AccountKey),
    AccountFlameConfigHasAlreadyEpheremallyUpdated(AccountKey),
}

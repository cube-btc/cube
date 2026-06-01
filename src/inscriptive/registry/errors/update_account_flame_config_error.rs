/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with updating an account's flame config.
#[derive(Debug, Clone)]
pub enum RMUpdateAccountFlameConfigError {
    AccountIsNotRegistered(AccountKey),
    AccountFlameConfigHasAlreadyEpheremallyUpdated(AccountKey),
}

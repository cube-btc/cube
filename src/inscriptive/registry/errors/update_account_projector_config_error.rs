/// Account Key.
type AccountKey = [u8; 32];

/// Projector config key of an account.
type AccountProjectorConfig = [u8; 32];

/// Errors associated with updating an account's projector config.
#[derive(Debug, Clone)]
pub enum RMUpdateAccountProjectorConfigError {
    AccountIsNotRegistered(AccountKey),
    ProjectorConfigIsAlreadyEpheremallyUpdated(AccountKey, AccountProjectorConfig),
}

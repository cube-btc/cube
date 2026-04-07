/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with updating the last activity timestamp of an account.
#[derive(Debug, Clone)]
pub enum RMUpdateAccountLastActivityTimestampError {
    AccountIsNotRegistered(AccountKey),
}

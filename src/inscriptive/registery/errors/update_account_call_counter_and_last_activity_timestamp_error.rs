/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with updating the call counter and last activity timestamp of an account.
#[derive(Debug, Clone)]
pub enum RMUpdateAccountCallCounterAndLastActivityTimestampError {
    AccountIsNotRegistered(AccountKey),
}

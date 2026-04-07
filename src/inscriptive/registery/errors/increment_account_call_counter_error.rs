/// Account Key.
type AccountKey = [u8; 32];

/// Errors associated with incrementing the call counter of an account.
#[derive(Debug, Clone)]
pub enum RMIncrementAccountCallCounterError {
    AccountIsNotRegistered(AccountKey),
}

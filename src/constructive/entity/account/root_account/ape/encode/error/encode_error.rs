/// Type alias for the account key.
type AccountKey = [u8; 32];

/// Enum to represent errors that can occur when encoding a `RootAccount` as a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RootAccountAPEEncodeError {
    UnableToRetrieveRankValueFromRegisteryManager(AccountKey),
}

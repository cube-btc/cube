/// Type alias for the account key.
type AccountKey = [u8; 32];

/// Enum to represent errors that can occur when encoding an `Account` as a bit vector.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountAPEEncodeError {
    RankNotFoundError(AccountKey),
}

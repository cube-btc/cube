/// Account Key.
type AccountKey = [u8; 32];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type AccountSecondaryAggregationKey = Vec<u8>;

/// Errors associated with updating an account's secondary aggregation key.
#[derive(Debug, Clone)]
pub enum RMUpdateAccountSecondaryAggregationKeyError {
    AccountIsNotRegistered(AccountKey),
    SecondaryAggregationKeyIsAlreadyEpheremallyUpdated(AccountKey, AccountSecondaryAggregationKey),
}

use crate::inscriptive::registry::errors::update_account_call_counter_and_last_activity_timestamp_error::RMUpdateAccountCallCounterAndLastActivityTimestampError;
use crate::inscriptive::registry::errors::update_account_bls_key_error::RMUpdateAccountBLSKeyError;

/// Errors associated with syncing a `RegisteredButUnconfiguredRootAccount` with the `Registry`.
#[derive(Debug, Clone)]
pub enum RegisteredButUnconfiguredRootAccountSyncWithRegistryError {
    RegistryUpdateAccountCallCounterAndLastActivityTimestampError(
        RMUpdateAccountCallCounterAndLastActivityTimestampError,
    ),
    RegistrySetAccountBLSKeyError(RMUpdateAccountBLSKeyError),
}

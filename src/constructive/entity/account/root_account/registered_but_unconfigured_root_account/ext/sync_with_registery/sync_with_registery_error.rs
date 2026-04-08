use crate::inscriptive::registery::errors::update_account_call_counter_and_last_activity_timestamp_error::RMUpdateAccountCallCounterAndLastActivityTimestampError;
use crate::inscriptive::registery::errors::update_account_bls_key_error::RMUpdateAccountBLSKeyError;

/// Errors associated with syncing a `RegisteredButUnconfiguredRootAccount` with the `Registery`.
#[derive(Debug, Clone)]
pub enum RegisteredButUnconfiguredRootAccountSyncWithRegisteryError {
    RegisteryUpdateAccountCallCounterAndLastActivityTimestampError(
        RMUpdateAccountCallCounterAndLastActivityTimestampError,
    ),
    RegisterySetAccountBLSKeyError(RMUpdateAccountBLSKeyError),
}

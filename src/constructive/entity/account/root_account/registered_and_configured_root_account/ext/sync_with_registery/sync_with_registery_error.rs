use crate::inscriptive::registery::errors::increment_account_call_counter_error::RMIncrementAccountCallCounterError;
use crate::inscriptive::registery::errors::update_account_last_activity_timestamp_error::RMUpdateAccountLastActivityTimestampError;

/// Errors associated with syncing a `RegisteredAndConfiguredRootAccount` with the `Registery`.
#[derive(Debug, Clone)]
pub enum RegisteredAndConfiguredRootAccountSyncWithRegisteryError {
    RegisteryIncrementAccountCallCounterError(RMIncrementAccountCallCounterError),
    RegisteryUpdateAccountLastActivityTimestampError(RMUpdateAccountLastActivityTimestampError),
}
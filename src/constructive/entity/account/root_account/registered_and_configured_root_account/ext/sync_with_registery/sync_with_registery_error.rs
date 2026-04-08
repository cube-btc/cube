use crate::inscriptive::registery::errors::update_account_call_counter_and_last_activity_timestamp_error::RMUpdateAccountCallCounterAndLastActivityTimestampError;

/// Errors associated with syncing a `RegisteredAndConfiguredRootAccount` with the `Registery`.
#[derive(Debug, Clone)]
pub enum RegisteredAndConfiguredRootAccountSyncWithRegisteryError {
    RegisteryUpdateAccountCallCounterAndLastActivityTimestampError(RMUpdateAccountCallCounterAndLastActivityTimestampError),
}
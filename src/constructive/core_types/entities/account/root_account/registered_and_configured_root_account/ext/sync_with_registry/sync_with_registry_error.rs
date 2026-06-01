use crate::inscriptive::registry::errors::update_account_call_counter_and_last_activity_timestamp_error::RMUpdateAccountCallCounterAndLastActivityTimestampError;

/// Errors associated with syncing a `RegisteredAndConfiguredRootAccount` with the `Registry`.
#[derive(Debug, Clone)]
pub enum RegisteredAndConfiguredRootAccountSyncWithRegistryError {
    RegistryUpdateAccountCallCounterAndLastActivityTimestampError(RMUpdateAccountCallCounterAndLastActivityTimestampError),
}
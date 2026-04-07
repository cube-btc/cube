use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredAndConfiguredRootAccountSyncWithRegisteryError;

impl RegisteredAndConfiguredRootAccount {
    pub async fn sync_with_registery(
        &self,
        session_timestamp: u64,
        registery: &REGISTERY,
        optimized: bool,
    ) -> Result<(), RegisteredAndConfiguredRootAccountSyncWithRegisteryError> {
        // 1 Lock the registery.
        let mut _registery = registery.lock().await;

        // 2 Increment the call counter.
        _registery
            .increment_account_call_counter_by_one(self.account_key, optimized)
            .map_err(|e| {
                RegisteredAndConfiguredRootAccountSyncWithRegisteryError::RegisteryIncrementAccountCallCounterError(e)
            })?;

        // 3 Update the last activity timestamp.
        _registery
            .update_account_last_activity_timestamp(self.account_key, session_timestamp, optimized)
            .map_err(|e| {
                RegisteredAndConfiguredRootAccountSyncWithRegisteryError::RegisteryUpdateAccountLastActivityTimestampError(
                    e,
                )
            })?;

        // 4 Return Ok.
        Ok(())
    }
}

use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredAndConfiguredRootAccountSyncWithRegisteryError;

impl RegisteredAndConfiguredRootAccount {
    pub async fn sync_with_registery(
        &self,
        execution_timestamp: u64,
        registery: &REGISTERY,
    ) -> Result<(), RegisteredAndConfiguredRootAccountSyncWithRegisteryError> {
        // 1 Lock the registery.
        let mut _registery = registery.lock().await;

        // 2 Update the call counter and last activity timestamp.
        _registery
            .update_account_call_counter_and_last_activity_timestamp(self.account_key, execution_timestamp)
            .map_err(|e| {
                RegisteredAndConfiguredRootAccountSyncWithRegisteryError::RegisteryUpdateAccountCallCounterAndLastActivityTimestampError(e)
            })?;

        // 3 Return Ok.
        Ok(())
    }
}

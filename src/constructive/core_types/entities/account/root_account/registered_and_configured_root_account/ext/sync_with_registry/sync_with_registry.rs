use crate::constructive::entity::account::root_account::registered_and_configured_root_account::registered_and_configured_root_account::RegisteredAndConfiguredRootAccount;
use crate::inscriptive::registry::registry::REGISTRY;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredAndConfiguredRootAccountSyncWithRegistryError;

impl RegisteredAndConfiguredRootAccount {
    pub async fn sync_with_registry(
        &self,
        execution_timestamp: u64,
        registry: &REGISTRY,
    ) -> Result<(), RegisteredAndConfiguredRootAccountSyncWithRegistryError> {
        // 1 Lock the registry.
        let mut _registry = registry.lock().await;

        // 2 Update the call counter and last activity timestamp.
        _registry
            .update_account_call_counter_and_last_activity_timestamp(self.account_key, execution_timestamp)
            .map_err(|e| {
                RegisteredAndConfiguredRootAccountSyncWithRegistryError::RegistryUpdateAccountCallCounterAndLastActivityTimestampError(e)
            })?;

        // 3 Return Ok.
        Ok(())
    }
}

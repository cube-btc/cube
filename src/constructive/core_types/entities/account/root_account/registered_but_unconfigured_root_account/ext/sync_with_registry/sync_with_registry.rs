use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::inscriptive::registry::registry::REGISTRY;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredButUnconfiguredRootAccountSyncWithRegistryError;

impl RegisteredButUnconfiguredRootAccount {
    pub async fn sync_with_registry(
        &self,
        execution_timestamp: u64,
        registry: &REGISTRY,
    ) -> Result<(), RegisteredButUnconfiguredRootAccountSyncWithRegistryError> {
        // 1 Get BLS key to be configured.
        let bls_key_to_be_configured = self.bls_key_to_be_configured;

        // 2 Lock the registry.
        let mut _registry = registry.lock().await;

        // 3 Update the call counter and last activity timestamp.
        _registry
            .update_account_call_counter_and_last_activity_timestamp(self.account_key, execution_timestamp)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegistryError::RegistryUpdateAccountCallCounterAndLastActivityTimestampError(e)
            })?;

        // 4 Update the BLS key.
        _registry
            .set_account_bls_key(self.account_key, bls_key_to_be_configured)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegistryError::RegistrySetAccountBLSKeyError(e)
            })?;

        // 5 Return Ok.
        Ok(())
    }
}

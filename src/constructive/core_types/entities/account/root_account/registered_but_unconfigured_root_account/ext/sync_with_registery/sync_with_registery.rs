use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError;

impl RegisteredButUnconfiguredRootAccount {
    pub async fn sync_with_registery(
        &self,
        execution_timestamp: u64,
        registery: &REGISTERY,
    ) -> Result<(), RegisteredButUnconfiguredRootAccountSyncWithRegisteryError> {
        // 1 Get BLS key to be configured.
        let bls_key_to_be_configured = self.bls_key_to_be_configured;

        // 2 Lock the registery.
        let mut _registery = registery.lock().await;

        // 3 Update the call counter and last activity timestamp.
        _registery
            .update_account_call_counter_and_last_activity_timestamp(self.account_key, execution_timestamp)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegisteryError::RegisteryUpdateAccountCallCounterAndLastActivityTimestampError(e)
            })?;

        // 4 Update the BLS key.
        _registery
            .set_account_bls_key(self.account_key, bls_key_to_be_configured)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegisteryError::RegisterySetAccountBLSKeyError(e)
            })?;

        // 5 Return Ok.
        Ok(())
    }
}

use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::registered_but_unconfigured_root_account::RegisteredButUnconfiguredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError;

impl RegisteredButUnconfiguredRootAccount {
    pub async fn sync_with_registery(
        &self,
        session_timestamp: u64,
        registery: &REGISTERY,
        optimized: bool,
    ) -> Result<(), RegisteredButUnconfiguredRootAccountSyncWithRegisteryError> {
        // 1 Get BLS key to be configured.
        let bls_key_to_be_configured = self.bls_key_to_be_configured;

        // 2 Lock the registery.
        let mut _registery = registery.lock().await;

        // 3 Increment the call counter.
        _registery
            .increment_account_call_counter_by_one(self.account_key, optimized)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegisteryError::RegisteryIncrementAccountCallCounterError(e)
            })?;

        // 4 Update the last activity timestamp.
        _registery
            .update_account_last_activity_timestamp(self.account_key, session_timestamp, optimized)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegisteryError::RegisteryUpdateAccountLastActivityTimestampError(
                    e,
                )
            })?;

        // 5 Update the BLS key.
        _registery
            .set_account_bls_key(self.account_key, bls_key_to_be_configured, optimized)
            .map_err(|e| {
                RegisteredButUnconfiguredRootAccountSyncWithRegisteryError::RegisterySetAccountBLSKeyError(e)
            })?;

        // 6 Return Ok.
        Ok(())
    }
}

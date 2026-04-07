use crate::constructive::entity::account::root_account::unregistered_root_account::unregistered_root_account::UnregisteredRootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::sync_with_registery::sync_with_registery_error::UnregisteredRootAccountSyncWithRegisteryError;

impl UnregisteredRootAccount {
    pub async fn sync_with_registery(
        &self,
        session_timestamp: u64,
        registery: &REGISTERY,
        optimized: bool,
    ) -> Result<(), UnregisteredRootAccountSyncWithRegisteryError> {
        // 1 Register the account with the registery.
        {
            // 1.1 Lock the registery.
            let mut _registery = registery.lock().await;

            // 1.2 Register the account with the registery.
            _registery
                .register_account(
                    self.account_key_to_be_registered,
                    session_timestamp,
                    Some(self.bls_key_to_be_configured),
                    None,
                    self.flame_config_to_be_configured.clone(),
                    optimized,
                )
                .map_err(|e| {
                    UnregisteredRootAccountSyncWithRegisteryError::RegisteryRegisterAccountError(e)
                })?;
        }

        // 3 Return the result.
        Ok(())
    }
}

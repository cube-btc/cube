use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::constructive::entity::account::root_account::ext::sync_with_registery::sync_with_registery_error::RootAccountSyncWithRegisteryError;

impl RootAccount {
    pub async fn sync_with_registery(
        &self,
        session_timestamp: u64,
        registery: &REGISTERY,
        optimized: bool,
    ) -> Result<(), RootAccountSyncWithRegisteryError> {
        // 1 Match on the `RootAccount` type.
        match self {
            // 1.a The `RootAccount` is an `UnregisteredRootAccount`.
            Self::UnregisteredRootAccount(unregistered_root_account) => {
                unregistered_root_account.sync_with_registery(session_timestamp, registery, optimized).await.map_err(|e| {
                    RootAccountSyncWithRegisteryError::UnregisteredRootAccountSyncWithRegisteryError(e)
                })?;
            }
            // 1.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            Self::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                registered_but_unconfigured_root_account.sync_with_registery(session_timestamp, registery, optimized).await.map_err(|e| {
                    RootAccountSyncWithRegisteryError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(e)
                })?;
            }
            // 1.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            Self::RegisteredAndConfiguredRootAccount(registered_and_configured_root_account) => {
                registered_and_configured_root_account.sync_with_registery(session_timestamp, registery, optimized).await.map_err(|e| {
                    RootAccountSyncWithRegisteryError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError(e)
                })?;
            }
        }

        // 2 Return the result.
        Ok(())
    }
}

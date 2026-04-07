use crate::constructive::entity::account::root_account::unregistered_root_account::ext::sync_with_registery::sync_with_registery_error::UnregisteredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredAndConfiguredRootAccountSyncWithRegisteryError;

/// Errors associated with syncing a `RootAccount` with the `Registery`.
#[derive(Debug, Clone)]
pub enum RootAccountSyncWithRegisteryError {
    UnregisteredRootAccountSyncWithRegisteryError(UnregisteredRootAccountSyncWithRegisteryError),
    RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(RegisteredButUnconfiguredRootAccountSyncWithRegisteryError),
    RegisteredAndConfiguredRootAccountSyncWithRegisteryError(RegisteredAndConfiguredRootAccountSyncWithRegisteryError),
}
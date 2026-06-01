use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredAndConfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredButUnconfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::register_with_db::register_with_db_error::UnregisteredRootAccountRegisterWithDBError;
use crate::constructive::txo::lift::lift::Lift;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;

/// Errors associated with executing a `Liftup` entry.
#[derive(Debug, Clone)]
pub enum LiftupExecutionError {
    ValidateLiftV1ScriptpubkeyError(Lift),
    ValidateLiftV2ScriptpubkeyError(Lift),
    UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
    UnregisteredRootAccountInvalidAuthorizationSignatureError,
    UnregisteredRootAccountRegisterWithDBError(UnregisteredRootAccountRegisterWithDBError),
    RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
    RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
    RegisteredButUnconfiguredRootAccountSyncWithRegistryError(
        RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
    ),
    RegisteredAndConfiguredRootAccountSyncWithRegistryError(
        RegisteredAndConfiguredRootAccountSyncWithRegistryError,
    ),
    CoinManagerAccountBalanceUpError(CMAccountBalanceUpError),
    FailedToApplyFeesSubsidy,
}

use crate::constructive::entity::account::account::unregistered_account::ext::register_with_db::register_with_db_error::UnregisteredAccountRegisterWithDBError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredAndConfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registry::sync_with_registry_error::RegisteredButUnconfiguredRootAccountSyncWithRegistryError;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::register_with_db::register_with_db_error::UnregisteredRootAccountRegisterWithDBError;
use crate::inscriptive::coin_manager::errors::balance_update_errors::{
    CMAccountBalanceDownError, CMAccountBalanceUpError,
};

/// Errors associated with executing a `Move` entry.
#[derive(Debug, Clone)]
pub enum MoveExecutionError {
    /// `amount` plus post-subsidy entry fee does not fit in `u64`.
    MoveSenderTotalDebitOverflow,
    FromAndToAccountKeysAreSameError([u8; 32]),
    UnexpectedUnregisteredFromRootAccountError,
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
    UnregisteredToAccountValidateSchnorrKeyError,
    UnregisteredToAccountRegisterWithDBError(UnregisteredAccountRegisterWithDBError),
    CoinManagerAccountBalanceDownError(CMAccountBalanceDownError),
    CoinManagerAccountBalanceUpError(CMAccountBalanceUpError),

    /// Root account has no liveness flag in the privileges manager.
    RootAccountLivenessFlagNotFound([u8; 32]),
    /// Root account's ToBeFrozen takes-effect timestamp has elapsed.
    RootAccountFrozen([u8; 32]),
    /// Root account's ToBeDestroyed takes-effect timestamp has elapsed.
    RootAccountDestroyed([u8; 32]),

    FailedToApplyFeesSubsidy,
}

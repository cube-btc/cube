use crate::constructive::entity::account::account::unregistered_account::ext::register_with_db::register_with_db_error::UnregisteredAccountRegisterWithDBError;
use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredAndConfiguredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::register_with_db::register_with_db_error::UnregisteredRootAccountRegisterWithDBError;
use crate::inscriptive::coin_manager::errors::balance_update_errors::{
    CMAccountBalanceDownError, CMAccountBalanceUpError,
};

/// Errors associated with executing a `Move` entry.
#[derive(Debug, Clone)]
pub enum MoveExecutionError {
    AmountUnderflowAfterFeesError,
    UnexpectedUnregisteredFromRootAccountError,
    UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
    UnregisteredRootAccountInvalidAuthorizationSignatureError,
    UnregisteredRootAccountRegisterWithDBError(UnregisteredRootAccountRegisterWithDBError),
    RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
    RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
    RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(
        RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
    ),
    RegisteredAndConfiguredRootAccountSyncWithRegisteryError(
        RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
    ),
    UnregisteredToAccountValidateSchnorrKeyError,
    UnregisteredToAccountRegisterWithDBError(UnregisteredAccountRegisterWithDBError),
    CoinManagerAccountBalanceDownError(CMAccountBalanceDownError),
    CoinManagerAccountBalanceUpError(CMAccountBalanceUpError),
}

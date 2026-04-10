use crate::constructive::entity::account::root_account::registered_and_configured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredAndConfiguredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::registered_but_unconfigured_root_account::ext::sync_with_registery::sync_with_registery_error::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError;
use crate::constructive::entity::account::root_account::unregistered_root_account::ext::register_with_db::register_with_db_error::UnregisteredRootAccountRegisterWithDBError;
use crate::constructive::entry::entry_types::liftup::ext::validations::validate_lifts::validate_lifts_error::LiftupValidateLiftsError;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;

/// Errors associated with executing a `Liftup` entry.
#[derive(Debug, Clone)]
pub enum LiftupExecutionError {
    ValidateLiftsError(LiftupValidateLiftsError),
    UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
    UnregisteredRootAccountRegisterWithDBError(UnregisteredRootAccountRegisterWithDBError),
    RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
    RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(
        RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
    ),
    RegisteredAndConfiguredRootAccountSyncWithRegisteryError(
        RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
    ),
    CoinManagerAccountBalanceUpError(CMAccountBalanceUpError),
}

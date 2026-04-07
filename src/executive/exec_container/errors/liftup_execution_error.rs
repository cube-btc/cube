use crate::constructive::entry::entries::liftup::validate::validate_error::LiftupValidateError;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;
use crate::constructive::entity::account::root_account::ext::sync_with_registery::sync_with_registery_error::RootAccountSyncWithRegisteryError;
use crate::inscriptive::coin_manager::errors::register_errors::CMRegisterAccountError;
use crate::inscriptive::flame_manager::errors::register_account_error::FMRegisterAccountError;

/// Errors associated with executing a `Liftup` entry.
#[derive(Debug, Clone)]
pub enum LiftupExecutionError {
    LiftupValidationError(LiftupValidateError),
    RootAccountSyncWithRegisteryError(RootAccountSyncWithRegisteryError),
    CoinManagerRegisterAccountError(CMRegisterAccountError),
    CoinManagerIncreaseBalanceError(CMAccountBalanceUpError),
    FlameManagerRegisterAccountError(FMRegisterAccountError),
}

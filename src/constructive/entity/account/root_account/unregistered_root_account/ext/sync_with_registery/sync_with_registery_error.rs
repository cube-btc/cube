use crate::inscriptive::coin_manager::errors::register_errors::CMRegisterAccountError;
use crate::inscriptive::flame_manager::errors::register_account_error::FMRegisterAccountError;
use crate::inscriptive::registery::errors::register_account_error::RMRegisterAccountError;

/// Errors associated with syncing a `UnregisteredRootAccount` with the `Registery`.
#[derive(Debug, Clone)]
pub enum UnregisteredRootAccountSyncWithRegisteryError {
    RegisteryRegisterAccountError(RMRegisterAccountError),
    CoinManagerRegisterAccountError(CMRegisterAccountError),
    FlameManagerRegisterAccountError(FMRegisterAccountError),
}

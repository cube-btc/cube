use crate::inscriptive::coin_manager::errors::register_errors::CMRegisterAccountError;
use crate::inscriptive::flame_manager::errors::register_account_error::FMRegisterAccountError;
use crate::inscriptive::registery::errors::register_account_error::RMRegisterAccountError;

/// Errors associated with registering a `UnregisteredAccount` with the `DB`.
#[derive(Debug, Clone)]
pub enum UnregisteredAccountRegisterWithDBError {
    AccountHasBeenBurriedError,
    RegisteryRegisterAccountError(RMRegisterAccountError),
    CoinManagerRegisterAccountError(CMRegisterAccountError),
    FlameManagerRegisterAccountError(FMRegisterAccountError),
}

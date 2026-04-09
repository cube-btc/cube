use crate::constructive::entity::account::account::unregistered_account::unregistered_account::UnregisteredAccount;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::registery::registery::REGISTERY;
use crate::inscriptive::flame_manager::flame_manager::FLAME_MANAGER;
use crate::constructive::entity::account::account::unregistered_account::ext::register_with_db::register_with_db_error::UnregisteredAccountRegisterWithDBError;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;

impl UnregisteredAccount {
    /// Registers the `UnregisteredAccount` with the various database managers.
    pub async fn register_with_db(
        &self,
        session_timestamp: u64,
        registery: &REGISTERY,
        coin_manager: &COIN_MANAGER,
        flame_manager: &FLAME_MANAGER,
        graveyard: &GRAVEYARD,
        initial_account_balance_in_satoshis: u64,
    ) -> Result<(), UnregisteredAccountRegisterWithDBError> {
        // 1 Check if the Account has been burried.
        {
            // 1.1 Lock the graveyard.
            let _graveyard = graveyard.lock().await;

            // 1.2 Check if the account has already been burried.
            if _graveyard.is_account_burried(self.account_key_to_be_registered) {
                return Err(UnregisteredAccountRegisterWithDBError::AccountHasBeenBurriedError);
            }
        }

        // 2 Register the account with the registery.
        {
            // 2.1 Lock the registery.
            let mut _registery = registery.lock().await;

            // 2.2 Register the account with the registery.
            _registery
                .register_account(
                    self.account_key_to_be_registered,
                    session_timestamp,
                    None,
                    None,
                    None,
                )
                .map_err(|e| {
                    UnregisteredAccountRegisterWithDBError::RegisteryRegisterAccountError(e)
                })?;
        }

        // 3 Register the account with the `CoinManager`.
        {
            // 3.1 Lock the coin manager.
            let mut _coin_manager = coin_manager.lock().await;

            // 3.2 Register the `RootAccount` with the `CoinManager`.
            _coin_manager
                .register_account(
                    self.account_key_to_be_registered,
                    initial_account_balance_in_satoshis,
                )
                .map_err(|e| {
                    UnregisteredAccountRegisterWithDBError::CoinManagerRegisterAccountError(e)
                })?;
        }

        // 4 Register the account with the `FlameManager`.
        {
            // 3.1 Lock the flame manager.
            let mut _flame_manager = flame_manager.lock().await;

            // 3.2 Register the `RootAccount` with the `FlameManager`.
            _flame_manager
                .register_account(self.account_key_to_be_registered)
                .map_err(|e| {
                    UnregisteredAccountRegisterWithDBError::FlameManagerRegisterAccountError(e)
                })?;
        }

        // 5 Return the result.
        Ok(())
    }
}

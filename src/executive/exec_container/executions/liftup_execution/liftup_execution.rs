use crate::constructive::entry::entries::liftup::liftup::Liftup;
use crate::executive::exec_container::errors::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_container::exec_container::ExecContainer;

impl ExecContainer {
    /// Executes a `Liftup` entry.
    pub async fn execute_liftup_internal(
        &mut self,
        liftup: &Liftup,
        session_timestamp: u64,
        optimized: bool,
    ) -> Result<(), LiftupExecutionError> {
        // 1 Validate the `Liftup`.
        liftup
            .validate(
                self.engine_key,
                &self.utxo_set,
                &self.registery,
                &self.graveyard,
            )
            .await
            .map_err(|e| LiftupExecutionError::LiftupValidationError(e))?;

        // 2 Get the `RootAccount`.
        let root_account = &liftup.account;

        // 3 Get the Account's Schnorr key and BLS key.
        let (account_key, _bls_key, is_registered) = (
            root_account.account_key(),
            root_account.bls_key(),
            root_account.is_registered(),
        );

        // 4 Sync the `RootAccount` with the `Registery`.
        // This will under the hood register the `RootAccount` with the `Registery` if not registered.
        root_account
            .sync_with_registery(session_timestamp, &self.registery, optimized)
            .await
            .map_err(|e| LiftupExecutionError::RootAccountSyncWithRegisteryError(e))?;

        // 5 Register the Account with the `CoinManager` if not registered, or otherwise increase its balance.
        {
            // 5.1 Get the liftup sum value in satoshis.
            let liftup_sum_value_in_satoshis = liftup.liftup_sum_value_in_satoshis();

            // 5.2 Check whether the `RootAccount` is registered.
            match is_registered {
                // 5.2.a The `RootAccount` is not registered.
                false => {
                    // 5.2.a.1 Lock the coin manager.
                    let mut _coin_manager = self.coin_manager.lock().await;

                    // 5.2.a.2 Register the `RootAccount` with the `CoinManager`.
                    _coin_manager
                        .register_account(account_key, liftup_sum_value_in_satoshis)
                        .map_err(|e| LiftupExecutionError::CoinManagerRegisterAccountError(e))?;
                }
                // 5.2.b The `RootAccount` is registered.
                true => {
                    // 5.2.b.1 Lock the coin manager.
                    let mut _coin_manager = self.coin_manager.lock().await;

                    // 5.2.b.2 Increase the balance of the `RootAccount`.
                    _coin_manager
                        .account_balance_up(account_key, liftup_sum_value_in_satoshis)
                        .map_err(|e| LiftupExecutionError::CoinManagerIncreaseBalanceError(e))?;
                }
            }
        }

        // 6 Register the Account with the `FlameManager` if not registered.
        if !is_registered {
            // 6.1 Lock the flame manager.
            let mut _flame_manager = self.flame_manager.lock().await;

            // 6.2 Register the `RootAccount` with the `FlameManager`.
            _flame_manager
                .register_account(account_key)
                .map_err(|e| LiftupExecutionError::FlameManagerRegisterAccountError(e))?;
        }

        // 7 Return Ok.
        Ok(())
    }
}

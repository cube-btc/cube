use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;
use crate::executive::exec_container::errors::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_container::exec_container::ExecContainer;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;

impl ExecContainer {
    /// Executes a `Liftup` entry.
    pub async fn execute_liftup_internal(
        &mut self,
        liftup: &Liftup,
        session_timestamp: u64,
        validate_lifts_with_the_utxo_set: bool,
    ) -> Result<(), LiftupExecutionError> {
        // 1 Validate Lifts in the Liftup.
        liftup
            .validate_lifts(self.engine_key, &self.utxo_set, validate_lifts_with_the_utxo_set)
            .await
            .map_err(|e| LiftupExecutionError::ValidateLiftsError(e))?;

        // 2 Get the liftup sum value in satoshis.
        let liftup_sum_value_in_satoshis = liftup.liftup_sum_value_in_satoshis();

        // 3 Calculate fees.
        let fees: u64 = {
            // TODO
            0
        };

        // 4 Liftup value after fees.
        let liftup_value_after_fees_in_satoshis = liftup_sum_value_in_satoshis - fees;

        // 5 Get the `RootAccount`.
        let root_account = &liftup.root_account;

        // 6 Get the Account's Schnorr key and BLS key.
        let (_account_key, _bls_key, _is_registered) = (
            root_account.account_key(),
            root_account.bls_key(),
            root_account.is_registered(),
        );

        // 7 Match on the `RootAccount` type.
        match root_account {
            // 7.a The `RootAccount` is an `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 7.a.1 Validate the Schnorr and BLS keys are indeed valid.
                if !unregistered_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
                    );
                }

                // 7.a.2 Register the `UnregisteredRootAccount` with the `DB`.
                unregistered_root_account
                    .register_with_db(
                        session_timestamp,
                        &self.registery,
                        &self.coin_manager,
                        &self.flame_manager,
                        &self.graveyard,
                        liftup_value_after_fees_in_satoshis,
                    )
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::UnregisteredRootAccountRegisterWithDBError(e)
                    })?;
            }
            // 7.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 7.b.0 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(LiftupExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError);
                }

                // 7.b.1 Sync the `RegisteredButUnconfiguredRootAccount` with the `Registery`.
                registered_but_unconfigured_root_account.sync_with_registery(session_timestamp, &self.registery)
                    .await
                    .map_err(|e| LiftupExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(e))?;

                // 7.b.2 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(|e| LiftupExecutionError::CoinManagerAccountBalanceUpError(e))?;
            }
            // 7.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 7.c.1 Sync the `RegisteredAndConfiguredRootAccount` with the `Registery`.
                registered_and_configured_root_account.sync_with_registery(session_timestamp, &self.registery)
                    .await
                    .map_err(|e| LiftupExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError(e))?;

                // 7.c.2 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(|e| LiftupExecutionError::CoinManagerAccountBalanceUpError(e))?;
            }
        }

        // 8 Return Ok.
        Ok(())
    }
}

/// Increases the account balance with the `CoinManager`.
async fn increase_account_balance_with_coin_manager(
    coin_manager: &COIN_MANAGER,
    account_key: [u8; 32],
    amount: u64,
) -> Result<(), CMAccountBalanceUpError> {
    // 1 Lock the coin manager.
    let mut _coin_manager = coin_manager.lock().await;

    // 2 Increase the account balance.
    _coin_manager.account_balance_up(account_key, amount)
}

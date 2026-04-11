use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;

impl ExecCtx {
    /// Executes a `Liftup` entry.
    pub async fn execute_liftup_internal(
        &mut self,
        liftup: &Liftup,
        execution_timestamp: u64,
    ) -> Result<(), LiftupExecutionError> {
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

        // 6 Validate scriptpubkeys of the lifts.
        for lift in &liftup.lift_prevtxos { 
            // 6.1 Match on the lift type.
            match lift {
                // 6.1.a The lift is a `LiftV1`.
                Lift::LiftV1(liftv1) => {
                    // 6.1.a.1 Validate the scriptpubkey.
                    if !liftv1.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV1ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }

                // 6.1.b The lift is a `LiftV2`.
                Lift::LiftV2(liftv2) => {
                    // 6.1.b.1 Validate the scriptpubkey.
                    if !liftv2.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV2ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }

                // 6.1.c The lift is an unknown type.
                Lift::Unknown { .. } => {}
            }
        }

        // 7 Get the Account's Schnorr key and BLS key.
        let (_account_key, _bls_key, _is_registered) = (
            root_account.account_key(),
            root_account.bls_key(),
            root_account.is_registered(),
        );

        // 8 Match on the `RootAccount` type.
        match root_account {
            // 8.a The `RootAccount` is an `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 8.a.1 Validate the Schnorr and BLS keys are indeed valid.
                if !unregistered_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
                    );
                }

                // 8.a.2 Verify the BLS key authorization signature.
                if !unregistered_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 8.a.3 Register the `UnregisteredRootAccount` with the `DB`.
                unregistered_root_account
                    .register_with_db(
                        execution_timestamp,
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
            // 8.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 8.b.0 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 8.b.1 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 8.b.2 Sync the `RegisteredButUnconfiguredRootAccount` with the `Registery`.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(e)
                    })?;

                // 8.b.3 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
            // 8.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 8.c.1 Sync the `RegisteredAndConfiguredRootAccount` with the `Registery`.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError(e)
                    })?;

                // 8.c.2 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 9 Return Ok.
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

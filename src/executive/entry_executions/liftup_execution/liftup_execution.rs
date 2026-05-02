use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;

impl ExecCtx {
    /// Executes a `Liftup` entry.
    pub async fn execute_liftup_internal(
        &mut self,
        liftup: &Liftup,
        execution_timestamp: u64,
    ) -> Result<EntryFees, LiftupExecutionError> {
        // 1 Get the liftup sum value in satoshis.
        let liftup_sum_value_in_satoshis = liftup.liftup_sum_value_in_satoshis();

        // 2 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 3 Calculate fees.
        let base_fee = params_holder.liftup_entry_base_fee;
        let number_of_lifts = liftup.lift_tx_inputs.len() as u64;
        let per_lift_fee = number_of_lifts * params_holder.liftup_entry_per_lift_base_fee;
        let fees_pre_subsidy: u64 = base_fee + per_lift_fee;

        // 4 Get the latest consumption timestamp from the registery.
        let latest_consumption_timestamp = {
            let account_key = liftup.root_account.account_key();
            let _registery = self.registery.lock().await;
            _registery
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        // 5 Get fee exemptions for the account.
        let mut txfee_exemptions: Exemption = {
            let account_key = liftup.root_account.account_key();
            let _privileges_manager = self.privileges_manager.lock().unwrap();
            _privileges_manager
                .get_account_txfee_exemptions(account_key)
                .ok_or(LiftupExecutionError::FailedToGetAccountTxFeeExemptions(
                    account_key,
                ))
        }?;

        // 6 Apply the subsidy to the fees.
        let subsidy_breakdown = txfee_exemptions
            .apply_subsidy(
                execution_timestamp,
                latest_consumption_timestamp,
                fees_pre_subsidy,
            )
            .ok_or(LiftupExecutionError::FailedToApplyFeesSubsidy)?;

        // 7 Calculate the fees after subsidy.
        let fees_post_subsidy = subsidy_breakdown.post_discount_leftover;

        // 8 Calculate the liftup value after fees.
        let liftup_value_after_fees_in_satoshis = liftup_sum_value_in_satoshis - fees_post_subsidy;

        // 9 Get the `RootAccount`.
        let root_account = &liftup.root_account;

        // 10 Validate scriptpubkeys of the lifts.
        for lift in &liftup.lift_tx_inputs {
            // 10.1 Match on the lift type.
            match lift {
                // 10.1.a The lift is a `LiftV1`.
                Lift::LiftV1(liftv1) => {
                    // 10.1.a.1 Validate the scriptpubkey.
                    if !liftv1.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV1ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }

                // 10.1.b The lift is a `LiftV2`.
                Lift::LiftV2(liftv2) => {
                    // 10.1.b.1 Validate the scriptpubkey.
                    if !liftv2.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV2ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }

                // 10.1.c The lift is an unknown type.
                Lift::Unknown { .. } => {}
            }
        }

        // 11 Get the Account's Schnorr key and BLS key.
        let (_account_key, _bls_key, _is_registered) = (
            root_account.account_key(),
            root_account.bls_key(),
            root_account.is_registered(),
        );

        // 12 Match on the `RootAccount` type.
        match root_account {
            // 12.a The `RootAccount` is an `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                // 12.a.1 Validate the Schnorr and BLS keys are indeed valid.
                if !unregistered_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
                    );
                }

                // 12.a.2 Verify the BLS key authorization signature.
                if !unregistered_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 12.a.3 Register the `UnregisteredRootAccount` with the `DB`.
                unregistered_root_account
                    .register_with_db(
                        execution_timestamp,
                        &self.registery,
                        &self.coin_manager,
                        &self.flame_manager,
                        &self.privileges_manager,
                        &params_holder,
                        &self.graveyard,
                        liftup_value_after_fees_in_satoshis,
                    )
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::UnregisteredRootAccountRegisterWithDBError(e)
                    })?;
            }
            // 12.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 12.b.0 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 12.b.1 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 12.b.2 Sync the `RegisteredButUnconfiguredRootAccount` with the `Registery`.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError(e)
                    })?;

                // 12.b.3 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
            // 12.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 12.c.1 Sync the `RegisteredAndConfiguredRootAccount` with the `Registery`.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError(e)
                    })?;

                // 12.c.2 Increase the account balance with the `CoinManager`.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    _account_key,
                    liftup_value_after_fees_in_satoshis,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 13 Return Ok.
        Ok(EntryFees::Liftup {
            base_fee,
            per_lift_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
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

use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::txo::lift::lift::Lift;
use crate::executive::entry_executions::liftup_execution::error::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceUpError;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;

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

        // 3 Calculate nominal fees (pre-subsidy).
        let base_fee = params_holder.liftup_entry_base_fee;
        let number_of_lifts = liftup.lift_tx_inputs.len() as u64;
        let per_lift_fee = number_of_lifts * params_holder.liftup_entry_per_lift_base_fee;
        let fees_pre_subsidy: u64 = base_fee + per_lift_fee;

        // 4 Get the `RootAccount`.
        let root_account = &liftup.root_account;

        // 5 Validate scriptpubkeys of the lifts.
        for lift in &liftup.lift_tx_inputs {
            match lift {
                Lift::LiftV1(liftv1) => {
                    if !liftv1.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV1ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }
                Lift::LiftV2(liftv2) => {
                    if !liftv2.validate_scriptpubkey() {
                        return Err(LiftupExecutionError::ValidateLiftV2ScriptpubkeyError(
                            lift.clone(),
                        ));
                    }
                }
                Lift::Unknown { .. } => {}
            }
        }

        // 6 Account key (same for all `RootAccount` variants on this entry).
        let account_key = root_account.account_key();

        // 6.1 Last activity before this entry (read before `sync_with_registry`, which bumps it to `execution_timestamp`).
        let latest_activity_timestamp = {
            let _registry = self.registry.lock().await;
            _registry
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        // 7 Registry / registration first; fee subsidy only for registered roots and only when PM exemptions exist.
        let mut subsidy_breakdown: Option<ExemptionSubsidyBreakdown> = None;

        match root_account {
            RootAccount::UnregisteredRootAccount(unregistered_root_account) => {
                if !unregistered_root_account.validate_schnorr_and_bls_key() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountValidateSchnorrAndBLSKeyError,
                    );
                }

                if !unregistered_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::UnregisteredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                let liftup_value_after_fees_in_satoshis =
                    liftup_sum_value_in_satoshis.saturating_sub(fees_pre_subsidy);

                unregistered_root_account
                    .register_with_db(
                        execution_timestamp,
                        &self.registry,
                        &self.coin_manager,
                        &self.flame_manager,
                        &self.privileges_manager,
                        &params_holder,
                        &self.graveyard,
                        liftup_value_after_fees_in_satoshis,
                    )
                    .await
                    .map_err(LiftupExecutionError::UnregisteredRootAccountRegisterWithDBError)?;
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                registered_but_unconfigured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegistryError(e)
                    })?;

                let (fees_after_subsidy, bd) = self
                    .apply_subsidy_liftup(
                        account_key,
                        execution_timestamp,
                        fees_pre_subsidy,
                        latest_activity_timestamp,
                    )
                    .await?;

                subsidy_breakdown = bd;

                let liftup_credit_after_fees =
                    liftup_sum_value_in_satoshis.saturating_sub(fees_after_subsidy);

                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    account_key,
                    liftup_credit_after_fees,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                registered_and_configured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(|e| {
                        LiftupExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegistryError(e)
                    })?;

                let (fees_after_subsidy, bd) = self
                    .apply_subsidy_liftup(
                        account_key,
                        execution_timestamp,
                        fees_pre_subsidy,
                        latest_activity_timestamp,
                    )
                    .await?;

                subsidy_breakdown = bd;

                let liftup_credit_after_fees =
                    liftup_sum_value_in_satoshis.saturating_sub(fees_after_subsidy);

                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    account_key,
                    liftup_credit_after_fees,
                )
                .await
                .map_err(LiftupExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 8 Return Ok.
        Ok(EntryFees::Liftup {
            base_fee,
            per_lift_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
    }

    /// Effective liftup entry fee after subsidy (same first-return meaning as `apply_subsidy_move` / `apply_subsidy_swapout`).
    /// Root must already be synced — last activity, PM exemptions, `apply_subsidy`,
    /// ephemeral `set_or_update_account_txfee_exemptions` when subsidy ran (skipped if not permanently in PM).
    /// Caller subtracts this from `liftup_sum_value_in_satoshis` for the balance credit. Does not touch `CoinManager`.
    /// If the caller fails afterward, batch `rollback_last` restores PM and coin ephemerals together.
    async fn apply_subsidy_liftup(
        &self,
        account_key: [u8; 32],
        execution_timestamp: u64,
        fees_pre_subsidy: u64,
        latest_activity_timestamp: u64,
    ) -> Result<(u64, Option<ExemptionSubsidyBreakdown>), LiftupExecutionError> {
        let txfee_exemptions = {
            let _privileges_manager = self.privileges_manager.lock().await;
            _privileges_manager.get_account_txfee_exemptions(account_key)
        };

        let Some(mut exemptions) = txfee_exemptions else {
            return Ok((fees_pre_subsidy, None));
        };

        let bd = exemptions
            .apply_subsidy(
                execution_timestamp,
                latest_activity_timestamp,
                fees_pre_subsidy,
            )
            .ok_or(LiftupExecutionError::FailedToApplyFeesSubsidy)?;

        let fees_after_subsidy = bd.post_discount_leftover;

        {
            let mut _privileges_manager = self.privileges_manager.lock().await;
            match _privileges_manager
                .set_or_update_account_txfee_exemptions(account_key, exemptions)
            {
                Ok(()) => {}
                Err(PMUpdateAccountError::AccountIsNotPermanentlyRegistered(_)) => {
                    // Ephemeral PM row this batch: no delta write; fee math already used in-memory exemption.
                }
            }
        }

        Ok((fees_after_subsidy, Some(bd)))
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

use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::swapout::swapout::{Swapout, DUST_SWAPOUT_MIN};
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::executive::entry_executions::swapout_execution::error::swapout_execution_error::SwapoutExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;

impl ExecCtx {
    /// Executes a `Swapout` entry.
    pub async fn execute_swapout_internal(
        &mut self,
        swapout: &Swapout,
        execution_timestamp: u64,
    ) -> Result<EntryFees, SwapoutExecutionError> {
        // 1 Enforce swapout dust minimum.
        if swapout.amount < DUST_SWAPOUT_MIN {
            return Err(SwapoutExecutionError::SwapoutAmountBelowDustMin {
                amount: swapout.amount,
                dust_min: DUST_SWAPOUT_MIN,
            });
        }

        // 2 Validate the `PinlessSelf` scriptpubkey when it is `Default`.
        match &swapout.pinless_self {
            // 2.a Default pinless self.
            PinlessSelf::Default(pinless_self_default) => {
                // 2.a.1 Validate default scriptpubkey if location is present.
                if let Some(self_txout) = pinless_self_default.txout() {
                    // 2.a.1.1 Get the calculated scriptpubkey.
                    let calculated_scriptpubkey =
                    match pinless_self_default.calculated_scriptpubkey() {
                        Some(scriptpubkey) => scriptpubkey,
                        None => return Err(
                            SwapoutExecutionError::PinlessSelfDefaultFailedToGetCalculatedScriptpubkeyError,
                        ),
                    };

                    // 2.a.1.2 Validate the scriptpubkey.
                    if self_txout.script_pubkey.as_bytes() != calculated_scriptpubkey.as_slice() {
                        return Err(
                            SwapoutExecutionError::PinlessSelfDefaultScriptpubkeyMismatchError,
                        );
                    }
                }
            }
            // 2.b Unknown pinless self.
            PinlessSelf::Unknown(_) => {}
        }

        // 3 Sync/register sender root account with DB.
        match &swapout.root_account {
            // 3.a The `RootAccount` is an `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(SwapoutExecutionError::UnexpectedUnregisteredRootAccountError);
            }
            // 3.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 3.b.1 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 3.b.2 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 3.b.3 Sync with registery.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
            // 3.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 3.c.1 Sync with registery.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
        }

        // 4 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 5 Calculate the total debit amount (`amount + fee` after tx-fee subsidy).
        let base_fee = params_holder.swapout_entry_base_fee;
        let fees_pre_subsidy = base_fee;

        // 6 Get the account key.
        let account_key = swapout.root_account.account_key();

        // 7 Get the latest consumption timestamp.
        let latest_consumption_timestamp = {
            let _registery = self.registery.lock().await;
            _registery
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        // 8 Get the tx-fee exemptions.
        let mut txfee_exemptions: Exemption = {
            let _privileges_manager = self.privileges_manager.lock().unwrap();
            _privileges_manager
                .get_account_txfee_exemptions(account_key)
                .ok_or(SwapoutExecutionError::FailedToGetAccountTxFeeExemptions(
                    account_key,
                ))?
        };

        // 9 Apply the subsidy to the fees.
        let subsidy_breakdown = txfee_exemptions
            .apply_subsidy(
                execution_timestamp,
                latest_consumption_timestamp,
                fees_pre_subsidy,
            )
            .ok_or(SwapoutExecutionError::FailedToApplyFeesSubsidy)?;

        // 10 Calculate the fees after subsidy.
        let fees_post_subsidy = subsidy_breakdown.post_discount_leftover;

        // 11 Calculate the total debit.
        let total_debit = u64::from(swapout.amount)
            .checked_add(fees_post_subsidy)
            .ok_or(SwapoutExecutionError::AmountPlusFeesOverflow)?;

        // 12 Decrease the root account balance with the `CoinManager`.
        {
            let mut _coin_manager = self.coin_manager.lock().await;
            _coin_manager
                .account_balance_down(swapout.root_account.account_key(), total_debit)
                .map_err(SwapoutExecutionError::CoinManagerAccountBalanceDownError)?;
        }

        // 13 Return the entry fees.
        Ok(EntryFees::Swapout {
            base_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
    }
}

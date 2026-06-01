use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::swapout::swapout::{Swapout, DUST_SWAPOUT_MIN};
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::executive::entry_executions::swapout_execution::error::swapout_execution_error::SwapoutExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::CMAccountBalanceDownError;
use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;

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
            PinlessSelf::Default(pinless_self_default) => {
                if let Some(self_txout) = pinless_self_default.txout() {
                    let calculated_scriptpubkey = match pinless_self_default
                        .calculated_scriptpubkey()
                    {
                        Some(scriptpubkey) => scriptpubkey,
                        None => {
                            return Err(
                                    SwapoutExecutionError::PinlessSelfDefaultFailedToGetCalculatedScriptpubkeyError,
                                );
                        }
                    };

                    if self_txout.script_pubkey.as_bytes() != calculated_scriptpubkey.as_slice() {
                        return Err(
                            SwapoutExecutionError::PinlessSelfDefaultScriptpubkeyMismatchError,
                        );
                    }
                }
            }
            PinlessSelf::Unknown(_) => {}
        }

        // 3 Params and nominal entry fee (pre-subsidy); account key for subsidy reads.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };
        let base_fee = params_holder.swapout_entry_base_fee;
        let fees_pre_subsidy = base_fee;
        let account_key = swapout.root_account.account_key();

        // 3.1 Last activity before this entry (read before `sync_with_registry`, which bumps it to `execution_timestamp`).
        let latest_activity_timestamp = {
            let _registry = self.registry.lock().await;
            _registry
                .get_account_last_activity_timestamp(account_key)
                .unwrap_or(0)
        };

        // 4 `RootAccount`: sync then tx-fee subsidy (registry / PM; no `CoinManager` here).
        let (fees_after_subsidy, subsidy_breakdown) = match &swapout.root_account {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(SwapoutExecutionError::UnexpectedUnregisteredRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                registered_but_unconfigured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
                    )?;

                self.apply_subsidy_swapout(
                    account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                    .await?
            }
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                registered_and_configured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegistryError,
                    )?;

                self.apply_subsidy_swapout(
                    account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                    .await?
            }
        };

        // 5 Total debit (`amount` + fee after subsidy).
        let total_debit = u64::from(swapout.amount)
            .checked_add(fees_after_subsidy)
            .ok_or(SwapoutExecutionError::AmountPlusFeesOverflow)?;

        // 6 Decrease the root account balance with the `CoinManager`.
        decrease_account_balance_with_coin_manager(&self.coin_manager, account_key, total_debit)
            .await
            .map_err(SwapoutExecutionError::CoinManagerAccountBalanceDownError)?;

        // 7 Return the entry fees.
        Ok(EntryFees::Swapout {
            base_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
    }

    /// Applies the subsidy to the swapout entry fees.
    async fn apply_subsidy_swapout(
        &self,
        account_key: [u8; 32],
        execution_timestamp: u64,
        fees_pre_subsidy: u64,
        latest_activity_timestamp: u64,
    ) -> Result<(u64, Option<ExemptionSubsidyBreakdown>), SwapoutExecutionError> {
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
            .ok_or(SwapoutExecutionError::FailedToApplyFeesSubsidy)?;

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

async fn decrease_account_balance_with_coin_manager(
    coin_manager: &COIN_MANAGER,
    account_key: [u8; 32],
    amount: u64,
) -> Result<(), CMAccountBalanceDownError> {
    let mut _coin_manager = coin_manager.lock().await;
    _coin_manager.account_balance_down(account_key, amount)
}

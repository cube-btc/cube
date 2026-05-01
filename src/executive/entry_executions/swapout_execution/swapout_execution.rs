use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::swapout::swapout::{Swapout, DUST_SWAPOUT_MIN};
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::executive::entry_executions::swapout_execution::error::swapout_execution_error::SwapoutExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;

impl ExecCtx {
    /// Executes a `Swapout` entry.
    pub async fn execute_swapout_internal(
        &mut self,
        swapout: &Swapout,
        execution_timestamp: u64,
    ) -> Result<EntryFees, SwapoutExecutionError> {
        // 0 Enforce swapout dust minimum.
        if swapout.amount < DUST_SWAPOUT_MIN {
            return Err(SwapoutExecutionError::SwapoutAmountBelowDustMin {
                amount: swapout.amount,
                dust_min: DUST_SWAPOUT_MIN,
            });
        }

        // 1 Validate the `PinlessSelf` scriptpubkey when it is `Default`.
        match &swapout.pinless_self {
            // 1.a Default pinless self.
            PinlessSelf::Default(pinless_self_default) => {
                // 1.a.1 Validate default scriptpubkey if location is present.
                if let Some(self_txout) = pinless_self_default.txout() {
                    // 1.a.1.1 Get the calculated scriptpubkey.
                    let calculated_scriptpubkey =
                    match pinless_self_default.calculated_scriptpubkey() {
                        Some(scriptpubkey) => scriptpubkey,
                        None => return Err(
                            SwapoutExecutionError::PinlessSelfDefaultFailedToGetCalculatedScriptpubkeyError,
                        ),
                    };

                    // 1.a.1.2 Validate the scriptpubkey.
                    if self_txout.script_pubkey.as_bytes() != calculated_scriptpubkey.as_slice() {
                        return Err(
                            SwapoutExecutionError::PinlessSelfDefaultScriptpubkeyMismatchError,
                        );
                    }
                }
            }
            // 1.b Unknown pinless self.
            PinlessSelf::Unknown(_) => {}
        }

        // 2 Sync/register sender root account with DB.
        match &swapout.root_account {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(SwapoutExecutionError::UnexpectedUnregisteredRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 2.b.1 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 2.b.2 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 2.b.3 Sync with registery.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 2.c.1 Sync with registery.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        SwapoutExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
        }

        // 3 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 4 Calculate the total debit amount (`amount + base_fee`).
        // 4.1 Get the swapout base fee.
        let base_fee = params_holder.swapout_entry_base_fee;

        // 4.2 Calculate total debit.
        let total_debit = u64::from(swapout.amount)
            .checked_add(base_fee)
            .ok_or(SwapoutExecutionError::AmountPlusFeesOverflow)?;

        // 5 Decrease the root account balance with the `CoinManager`.
        {
            let mut _coin_manager = self.coin_manager.lock().await;
            _coin_manager
                .account_balance_down(swapout.root_account.account_key(), total_debit)
                .map_err(SwapoutExecutionError::CoinManagerAccountBalanceDownError)?;
        }

        // 6 Return the entry fees.
        Ok(EntryFees::Swapout {
            base_fee,
            total: base_fee,
        })
    }
}

use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::executive::entry_executions::move_execution::error::move_execution_error::MoveExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::{
    CMAccountBalanceDownError, CMAccountBalanceUpError,
};
use crate::inscriptive::privileges_manager::elements::exemption::exemption::Exemption;

impl ExecCtx {
    /// Executes a `Move` entry.
    pub async fn execute_move_internal(
        &mut self,
        move_entry: &Move,
        execution_timestamp: u64,
    ) -> Result<EntryFees, MoveExecutionError> {
        // 1 Get move amount in satoshis.
        let move_amount_in_satoshis = move_entry.amount as u64;

        // 2 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 3 Calculate fees.
        let base_fee = params_holder.move_entry_base_fee;
        let liquidity_fee =
            (move_amount_in_satoshis * params_holder.move_ppm_liquidity_fee) / 1_000_000;
        let fees_pre_subsidy: u64 = base_fee + liquidity_fee;

        // 4 Resolve sender and receiver account keys (sender pays tx-fee subsidy).
        let from_account_key = move_entry.from.account_key();
        let to_account_key = move_entry.to.account_key();

        // 5 Last activity timestamp from the registery (periodic fee-credit consumption).
        let latest_consumption_timestamp = {
            let _registery = self.registery.lock().await;
            _registery
                .get_account_last_activity_timestamp(from_account_key)
                .unwrap_or(0)
        };

        // 6 Apply tx-fee exemptions for the sender.
        let mut txfee_exemptions: Exemption = {
            let _privileges_manager = self.privileges_manager.lock().unwrap();
            _privileges_manager
                .get_account_txfee_exemptions(from_account_key)
                .ok_or(MoveExecutionError::FailedToGetAccountTxFeeExemptions(
                    from_account_key,
                ))?
        };

        // 7 Apply the subsidy to the fees.
        let subsidy_breakdown = txfee_exemptions
            .apply_subsidy(
                execution_timestamp,
                latest_consumption_timestamp,
                fees_pre_subsidy,
            )
            .ok_or(MoveExecutionError::FailedToApplyFeesSubsidy)?;

        // 8 Calculate the fees after subsidy.
        let fees_post_subsidy = subsidy_breakdown.post_discount_leftover;

        // 9 Calculate the move value after fees.
        let move_value_after_fees_in_satoshis = move_amount_in_satoshis
            .checked_sub(fees_post_subsidy)
            .ok_or(MoveExecutionError::AmountUnderflowAfterFeesError)?;

        // 10 Reject self-transfer (`from` and `to` keys must be different).
        if from_account_key == to_account_key {
            return Err(MoveExecutionError::FromAndToAccountKeysAreSameError(
                from_account_key,
            ));
        }

        // 11 Sync/register sender root account with DB.
        match &move_entry.from {
            // 11.a The `RootAccount` is an `UnregisteredRootAccount`.
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(MoveExecutionError::UnexpectedUnregisteredFromRootAccountError);
            }
            // 11.b The `RootAccount` is a `RegisteredButUnconfiguredRootAccount`.
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 11.b.1 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 11.b.2 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 11.b.3 Sync with registery.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
            // 11.c The `RootAccount` is a `RegisteredAndConfiguredRootAccount`.
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 11.c.1 Sync with registery.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        MoveExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
        }

        // 12 Decrease sender balance in the coin manager first.
        decrease_account_balance_with_coin_manager(
            &self.coin_manager,
            from_account_key,
            move_amount_in_satoshis,
        )
        .await
        .map_err(MoveExecutionError::CoinManagerAccountBalanceDownError)?;

        // 13 Sync/register receiver account with DB.
        match &move_entry.to {
            Account::UnregisteredAccount(unregistered_account) => {
                // 13.a.1 Validate Schnorr key.
                if !unregistered_account.validate_schnorr_key() {
                    return Err(MoveExecutionError::UnregisteredToAccountValidateSchnorrKeyError);
                }

                // 13.a.2 Register receiver account with DB using move value after fees as initial balance.
                unregistered_account
                    .register_with_db(
                        execution_timestamp,
                        &self.registery,
                        &self.coin_manager,
                        &self.flame_manager,
                        &self.privileges_manager,
                        &params_holder,
                        &self.graveyard,
                        move_value_after_fees_in_satoshis,
                    )
                    .await
                    .map_err(MoveExecutionError::UnregisteredToAccountRegisterWithDBError)?;
            }
            Account::RegisteredAccount(_) => {
                // 13.b.1 Receiver is already registered; credit via coin manager.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    to_account_key,
                    move_value_after_fees_in_satoshis,
                )
                .await
                .map_err(MoveExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 14 Return Ok.
        Ok(EntryFees::Move {
            base_fee,
            liquidity_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
    }
}

/// Decreases the account balance with the `CoinManager`.
async fn decrease_account_balance_with_coin_manager(
    coin_manager: &COIN_MANAGER,
    account_key: [u8; 32],
    amount: u64,
) -> Result<(), CMAccountBalanceDownError> {
    let mut _coin_manager = coin_manager.lock().await;
    _coin_manager.account_balance_down(account_key, amount)
}

/// Increases the account balance with the `CoinManager`.
async fn increase_account_balance_with_coin_manager(
    coin_manager: &COIN_MANAGER,
    account_key: [u8; 32],
    amount: u64,
) -> Result<(), CMAccountBalanceUpError> {
    let mut _coin_manager = coin_manager.lock().await;
    _coin_manager.account_balance_up(account_key, amount)
}

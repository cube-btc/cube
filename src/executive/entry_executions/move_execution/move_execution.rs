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
use crate::inscriptive::privileges_manager::elements::exemption::exemption::ExemptionSubsidyBreakdown;
use crate::inscriptive::privileges_manager::errors::update_error::PMUpdateAccountError;

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

        // 3 Calculate nominal fees (pre-subsidy).
        let base_fee = params_holder.move_entry_base_fee;
        let liquidity_fee =
            (move_amount_in_satoshis * params_holder.move_ppm_liquidity_fee) / 1_000_000;
        let fees_pre_subsidy: u64 = base_fee + liquidity_fee;

        // 4 Resolve sender and receiver account keys (sender pays tx-fee subsidy).
        let from_account_key = move_entry.from.account_key();
        let to_account_key = move_entry.to.account_key();

        // 5 Reject self-transfer (`from` and `to` keys must be different).
        if from_account_key == to_account_key {
            return Err(MoveExecutionError::FromAndToAccountKeysAreSameError(
                from_account_key,
            ));
        }

        // 5.1 Last activity before this entry (read before `sync_with_registry`, which bumps it to `execution_timestamp`).
        let latest_activity_timestamp = {
            let _registry = self.registry.lock().await;
            _registry
                .get_account_last_activity_timestamp(from_account_key)
                .unwrap_or(0)
        };

        // 6 `RootAccount::from`: sync then sender tx-fee subsidy (registry / PM; no `CoinManager` here).
        let (fees_after_subsidy, subsidy_breakdown) = match &move_entry.from {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(MoveExecutionError::UnexpectedUnregisteredFromRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                registered_but_unconfigured_root_account
                    .sync_with_registry(execution_timestamp, &self.registry)
                    .await
                    .map_err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegistryError,
                    )?;

                self.apply_subsidy_move(
                    from_account_key,
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
                        MoveExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegistryError,
                    )?;

                self.apply_subsidy_move(
                    from_account_key,
                    execution_timestamp,
                    fees_pre_subsidy,
                    latest_activity_timestamp,
                )
                    .await?
            }
        };

        // 7 Receiver gets the full move `amount`; sender pays `amount` plus post-subsidy entry fees.
        let sender_total_debit = move_amount_in_satoshis
            .checked_add(fees_after_subsidy)
            .ok_or(MoveExecutionError::MoveSenderTotalDebitOverflow)?;

        // 8 Decrease sender balance (`amount` + fees) before crediting the receiver.
        decrease_account_balance_with_coin_manager(
            &self.coin_manager,
            from_account_key,
            sender_total_debit,
        )
        .await
        .map_err(MoveExecutionError::CoinManagerAccountBalanceDownError)?;

        // 9 Sync/register receiver account with DB.
        match &move_entry.to {
            Account::UnregisteredAccount(unregistered_account) => {
                if !unregistered_account.validate_schnorr_key() {
                    return Err(MoveExecutionError::UnregisteredToAccountValidateSchnorrKeyError);
                }

                unregistered_account
                    .register_with_db(
                        execution_timestamp,
                        &self.registry,
                        &self.coin_manager,
                        &self.flame_manager,
                        &self.privileges_manager,
                        &params_holder,
                        &self.graveyard,
                        move_amount_in_satoshis,
                    )
                    .await
                    .map_err(MoveExecutionError::UnregisteredToAccountRegisterWithDBError)?;
            }
            Account::RegisteredAccount(_) => {
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    to_account_key,
                    move_amount_in_satoshis,
                )
                .await
                .map_err(MoveExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 10 Return Ok.
        Ok(EntryFees::Move {
            base_fee,
            liquidity_fee,
            total_pre_subsidy: fees_pre_subsidy,
            subsidy_breakdown,
        })
    }

    /// Applies the subsidy to the move entry fees.
    async fn apply_subsidy_move(
        &self,
        from_account_key: [u8; 32],
        execution_timestamp: u64,
        fees_pre_subsidy: u64,
        latest_activity_timestamp: u64,
    ) -> Result<(u64, Option<ExemptionSubsidyBreakdown>), MoveExecutionError> {
        let txfee_exemptions = {
            let _privileges_manager = self.privileges_manager.lock().await;
            _privileges_manager.get_account_txfee_exemptions(from_account_key)
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
            .ok_or(MoveExecutionError::FailedToApplyFeesSubsidy)?;

        let fees_after_subsidy = bd.post_discount_leftover;

        {
            let mut _privileges_manager = self.privileges_manager.lock().await;
            match _privileges_manager
                .set_or_update_account_txfee_exemptions(from_account_key, exemptions)
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

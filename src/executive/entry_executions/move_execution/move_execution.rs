use crate::constructive::entity::account::account::account::Account;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::executive::entry_executions::move_execution::error::move_execution_error::MoveExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::coin_manager::errors::balance_update_errors::{
    CMAccountBalanceDownError, CMAccountBalanceUpError,
};

impl ExecCtx {
    /// Executes a `Move` entry.
    pub async fn execute_move_internal(
        &mut self,
        move_entry: &Move,
        execution_timestamp: u64,
    ) -> Result<(), MoveExecutionError> {
        // 1 Get move amount in satoshis.
        let move_amount_in_satoshis = move_entry.amount as u64;

        // 2 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 3 Calculate fees.
        let fees: u64 = {
            // 3.1 Get the base fee.
            let base_fee = params_holder.move_entry_base_fee;

            // 3.2 Calculate the proportional liquidity fee.
            let liquidity_fee =
                (move_amount_in_satoshis * params_holder.move_ppm_liquidity_fee) / 1_000_000;

            // 3.3 Calculate the total fee.
            let total_fee = base_fee + liquidity_fee;

            total_fee
        };

        // 4 Move value after fees.
        let move_value_after_fees_in_satoshis = move_amount_in_satoshis
            .checked_sub(fees)
            .ok_or(MoveExecutionError::AmountUnderflowAfterFeesError)?;

        // 4 Resolve sender and receiver account keys.
        let from_account_key = move_entry.from.account_key();
        let to_account_key = move_entry.to.account_key();

        // 4.1 Reject self-transfer (`from` and `to` keys must be different).
        if from_account_key == to_account_key {
            return Err(MoveExecutionError::FromAndToAccountKeysAreSameError(
                from_account_key,
            ));
        }

        // 5 Sync/register sender root account with DB.
        match &move_entry.from {
            RootAccount::UnregisteredRootAccount(_) => {
                return Err(MoveExecutionError::UnexpectedUnregisteredFromRootAccountError);
            }
            RootAccount::RegisteredButUnconfiguredRootAccount(
                registered_but_unconfigured_root_account,
            ) => {
                // 5.b.1 Validate the BLS key is indeed a valid BLS public key.
                if !registered_but_unconfigured_root_account.validate_bls_key() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountValidateBLSKeyError,
                    );
                }

                // 5.b.2 Verify the BLS key authorization signature.
                if !registered_but_unconfigured_root_account.verify_authorization_signature() {
                    return Err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountInvalidAuthorizationSignatureError,
                    );
                }

                // 5.b.3 Sync with registery.
                registered_but_unconfigured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        MoveExecutionError::RegisteredButUnconfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
            RootAccount::RegisteredAndConfiguredRootAccount(
                registered_and_configured_root_account,
            ) => {
                // 5.c.1 Sync with registery.
                registered_and_configured_root_account
                    .sync_with_registery(execution_timestamp, &self.registery)
                    .await
                    .map_err(
                        MoveExecutionError::RegisteredAndConfiguredRootAccountSyncWithRegisteryError,
                    )?;
            }
        }

        // 6 Decrease sender balance in the coin manager first.
        decrease_account_balance_with_coin_manager(
            &self.coin_manager,
            from_account_key,
            move_amount_in_satoshis,
        )
        .await
        .map_err(MoveExecutionError::CoinManagerAccountBalanceDownError)?;

        // 7 Sync/register receiver account with DB.
        match &move_entry.to {
            Account::UnregisteredAccount(unregistered_account) => {
                // 6.a.1 Validate Schnorr key.
                if !unregistered_account.validate_schnorr_key() {
                    return Err(MoveExecutionError::UnregisteredToAccountValidateSchnorrKeyError);
                }

                // 6.a.2 Register receiver account with DB using move value after fees as initial balance.
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
                // 6.b.1 Receiver is already registered; credit via coin manager.
                increase_account_balance_with_coin_manager(
                    &self.coin_manager,
                    to_account_key,
                    move_value_after_fees_in_satoshis,
                )
                .await
                .map_err(MoveExecutionError::CoinManagerAccountBalanceUpError)?;
            }
        }

        // 8 Return Ok.
        Ok(())
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

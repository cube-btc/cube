use crate::constructive::entry::entry_kinds::r#move::ext::pre_validations::validate_overall::validate_overall_error::MoveValidateOverallError;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use crate::inscriptive::graveyard::graveyard::GRAVEYARD;
use crate::inscriptive::registery::registery::REGISTERY;

impl Move {
    /// Used by the `Engine` to validate the `Move` end-to-end as a pre-validation step before executing it.
    pub async fn validate_overall(
        &self,
        execution_batch_height: u64,
        registery: &REGISTERY,
        graveyard: &GRAVEYARD,
        coin_manager: &COIN_MANAGER,
    ) -> Result<(), MoveValidateOverallError> {
        // 1 Reject unregistered sender root accounts for execution pre-validation.
        if matches!(&self.from, RootAccount::UnregisteredRootAccount(_)) {
            return Err(MoveValidateOverallError::UnregisteredRootAccountNotAllowedError);
        }

        // 2 Reject self-transfer (`from` and `to` keys must be different).
        let from_account_key = self.from.account_key();
        let to_account_key = self.to.account_key();
        if from_account_key == to_account_key {
            return Err(MoveValidateOverallError::FromAndToAccountKeysAreSameError(
                from_account_key,
            ));
        }

        // 3 Validate the sender root account.
        self.from
            .validate_root_account(registery, graveyard)
            .await
            .map_err(MoveValidateOverallError::ValidateRootAccountError)?;

        // 4 Validate the receiver account.
        self.to
            .validate_account(registery, graveyard)
            .await
            .map_err(MoveValidateOverallError::ValidateAccountError)?;

        // 5 Validate the target against the execution batch height.
        if let Err((targeted_at_batch_height, execution_batch_height_err)) =
            self.target.validate(execution_batch_height)
        {
            return Err(MoveValidateOverallError::ValidateTargetError {
                targeted_at_batch_height,
                execution_batch_height: execution_batch_height_err,
            });
        }

        // 6 Validate sender has enough balance to fund the move amount.
        {
            let required = self.amount as u64;

            // 6.1 Lock coin manager.
            let _coin_manager = coin_manager.lock().await;

            // 6.2 Resolve sender balance.
            let available = _coin_manager
                .get_account_balance(from_account_key)
                .ok_or(MoveValidateOverallError::FromAccountNotFoundInCoinManagerError(
                    from_account_key,
                ))?;

            // 6.3 Reject if sender balance is insufficient.
            if available < required {
                return Err(MoveValidateOverallError::InsufficientBalanceForMoveError {
                    account_key: from_account_key,
                    required,
                    available,
                });
            }
        }

        // 7 Ok.
        Ok(())
    }
}

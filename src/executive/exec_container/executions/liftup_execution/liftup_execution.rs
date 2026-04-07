use crate::constructive::entity::account::root_account::root_account::RootAccount;
use crate::constructive::entry::entries::liftup::liftup::Liftup;
use crate::executive::exec_container::errors::liftup_execution_error::LiftupExecutionError;
use crate::executive::exec_container::exec_container::ExecContainer;

impl ExecContainer {
    /// Executes a `Liftup` entry.
    pub async fn execute_liftup_internal(
        &mut self,
        liftup: &Liftup,
    ) -> Result<(), LiftupExecutionError> {
        // 1 Validate the liftup.
        if !liftup
            .validate(self.engine_key, &self.registery, &self.utxo_set)
            .await
        {
            return Err(LiftupExecutionError::LiftupValidationError);
        }

        // 2 Register the account if it's not registered.
        {
            match &liftup.account {
                RootAccount::UnregisteredRootAccount(unregistered_root_account) => {}
                RootAccount::RegisteredButUnconfiguredRootAccount(
                    registered_but_unconfigured_root_account,
                ) => {}
                RootAccount::RegisteredAndConfiguredRootAccount(
                    registered_and_configured_root_account,
                ) => {
                    //
                }
            }

            // 2 Liftup sum value in satoshis.
            let liftup_sum_value_in_satoshis = liftup.liftup_sum_value_in_satoshis();

            // 3 TODO: Fees management/privileges stuff.

            // 4 Increase the accounts balance.
            if let Err(error) = self
                .coin_manager
                .lock()
                .await
                .account_balance_up(liftup.account.account_key(), liftup_sum_value_in_satoshis)
            {
                return Err(LiftupExecutionError::CoinManagerIncreaseBalanceError(error));
            }

            // 2 Execute the liftup.
            Ok(())
        }
    }
}

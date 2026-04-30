use crate::constructive::entry::entry_fees::entry_fees::EntryFees;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::executive::entry_executions::swapout_execution::error::swapout_execution_error::SwapoutExecutionError;
use crate::executive::exec_ctx::exec_ctx::ExecCtx;

impl ExecCtx {
    /// Executes a `Swapout` entry.
    pub async fn execute_swapout_internal(
        &mut self,
        swapout: &Swapout,
        _execution_timestamp: u64,
    ) -> Result<EntryFees, SwapoutExecutionError> {
        // 1 Validate the `PinlessSelf` scriptpubkey when it is `Default`.
        match &swapout.pinless_self {
            PinlessSelf::Default(pinless_self_default) => {
                // 1.a Validate default scriptpubkey.
                if !pinless_self_default.validate_scriptpubkey() {
                    return Err(SwapoutExecutionError::ValidatePinlessSelfScriptpubkeyError);
                }
            }
            // 1.b Unknown pinless self is accepted without scriptpubkey validation.
            PinlessSelf::Unknown(_) => {}
        }

        // 2 Get params holder from the params manager.
        let params_holder = {
            let _params_manager = self._params_manager.lock().unwrap();
            _params_manager.get_params_holder()
        };

        // 3 Calculate the total debit amount (`amount + base_fee`).
        // 3.1 Get the swapout base fee.
        let base_fee = params_holder.swapout_entry_base_fee;
        // 3.2 Calculate total debit.
        let total_debit = u64::from(swapout.amount)
            .checked_add(base_fee)
            .ok_or(SwapoutExecutionError::AmountPlusFeesOverflow)?;

        // 4 Decrease the root account balance with the `CoinManager`.
        {
            let mut _coin_manager = self.coin_manager.lock().await;
            _coin_manager
                .account_balance_down(swapout.root_account.account_key(), total_debit)
                .map_err(SwapoutExecutionError::CoinManagerAccountBalanceDownError)?;
        }

        // 5 Return the entry fees.
        Ok(EntryFees::Swapout {
            base_fee,
            total: base_fee,
        })
    }
}

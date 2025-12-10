use crate::{
    constructive::entry::entries::call::call::Call,
    executive::{
        exec::{caller::Caller, exec::execute, exec_error::ExecutionError},
        stack::stack_item::StackItem,
    },
    inscriptive::{
        coin_manager::coin_manager::COIN_MANAGER,
        registery_manager::registery_manager::REGISTERY_MANAGER,
        state_manager::state_manager::STATE_MANAGER,
    },
};
use std::sync::Arc;

/// The type of the ops spent.
type OpsSpent = u32;

/// The type of the fees spent.
type FeesSpent = u32;

/// The context of a program execution.
pub struct ExecCtx {
    // The state holder.
    state_manager: STATE_MANAGER,
    // The coin holder.
    coin_manager: COIN_MANAGER,
    // The programs repo.
    registery_manager: REGISTERY_MANAGER,
    // External ops counter.
    external_ops_counter: u32,
    // The base ops price.
    base_ops_price: u32,
    // The timestamp.
    timestamp: u64,
    // Passed calls.
    passed_calls: Vec<(Call, OpsSpent, FeesSpent)>,
}

impl ExecCtx {
    /// Creates a new execution context.
    pub fn new(
        state_manager: &STATE_MANAGER,
        coin_manager: &COIN_MANAGER,
        registery_manager: &REGISTERY_MANAGER,
        base_ops_price: u32,
        timestamp: u64,
    ) -> Self {
        Self {
            state_manager: Arc::clone(state_manager),
            coin_manager: Arc::clone(coin_manager),
            registery_manager: Arc::clone(registery_manager),
            external_ops_counter: 0,
            base_ops_price,
            timestamp,
            passed_calls: Vec::<(Call, OpsSpent, FeesSpent)>::new(),
        }
    }

    /// Executes and inserts a call.
    pub async fn exec_insert_call(&mut self, call: Call) -> Result<(), ExecutionError> {
        // This is an external call.
        let internal = false;

        // The caller is the account key.
        let caller = Caller::new_account(call.account().account_key());

        // The contract id is the contract id of the called contract.
        let contract_id = call.contract().contract_id();

        // The method index is the method index of the called contract.
        let method_index = call.method_index();

        // Convert arg values to stack items.
        let args_as_stack_items = call
            .args()
            .iter()
            .map(|arg| arg.into_stack_item())
            .collect::<Vec<StackItem>>();

        // The timestamp is the timestamp of the execution context.
        let timestamp = self.timestamp;

        // The ops budget is the ops budget of the call.
        let ops_budget = call.ops_budget();

        // Check if the base ops price is the same as the base ops price of the call.
        if call.ops_price_base() != self.base_ops_price {
            return Err(ExecutionError::BaseOpsPriceMismatchError);
        }

        // The ops price is the total ops price of the call (base + extra).
        let ops_price = call.ops_price_total();

        // Internal ops counter is 0.
        let internal_ops_counter = 0;

        // External ops counter is the external ops counter of the call.
        let external_ops_counter = self.external_ops_counter;

        // State manager.
        let state_manager = &self.state_manager;

        // Pre-execution state backup.
        {
            let mut _state_manager = state_manager.lock().await;
            _state_manager.pre_execution();
        }

        let coin_manager = &self.coin_manager;

        // Pre-execution coin holder backup.
        {
            let mut _coin_manager = coin_manager.lock().await;
            _coin_manager.pre_execution();
        }

        // Programs repo.
        let registery_manager = &self.registery_manager;

        // Execution.
        let exectuion_result = execute(
            internal,
            caller,
            contract_id,
            method_index,
            args_as_stack_items,
            timestamp,
            ops_budget,
            ops_price,
            internal_ops_counter,
            external_ops_counter,
            state_manager,
            coin_manager,
            registery_manager,
        )
        .await;

        match exectuion_result {
            Ok((return_items, ops_spent, new_external_ops_counter)) => {
                // Stack must end with exactly one item and it must be true.
                match return_items.len() {
                    // Stack must end with exactly one item.
                    1 => {
                        // And that item must be exactly true.
                        if !return_items[0].is_true() {
                            return Err(ExecutionError::ReturnErrorFromStackError(
                                return_items[0].clone(),
                            ));
                        }
                    }
                    // If other than one item, return an error.
                    _ => {
                        return Err(ExecutionError::InvalidStackEndingError);
                    }
                }

                let fees_spent = ops_spent * self.base_ops_price;

                // Update the external ops counter.
                self.external_ops_counter = new_external_ops_counter;

                // Insert the call.
                self.passed_calls.push((call, ops_spent, fees_spent));

                // Return Ok.
                Ok(())
            }
            Err(error) => {
                // Rollback last on the registery manager.
                {
                    let mut _registery_manager = registery_manager.lock().await;
                    _registery_manager.rollback_last();
                }

                // Rollback last on the coin manager.
                {
                    let mut _coin_manager = coin_manager.lock().await;
                    _coin_manager.rollback_last();
                }

                // Rollback last on the state manager.
                {
                    let mut _state_manager = state_manager.lock().await;
                    _state_manager.rollback_last();
                }

                // Return the error.
                return Err(error);
            }
        }
    }

    /// Flushes all the passed calls.
    pub async fn flush_all(&mut self) {
        // Flush the registery manager delta.
        {
            let mut _registery_manager = self.registery_manager.lock().await;
            _registery_manager.flush_delta();
        }

        // Flush the coin manager delta.
        {
            let mut _coin_manager = self.coin_manager.lock().await;
            _coin_manager.flush_delta();
        }

        // Flush the state manager delta.
        {
            let mut _state_manager = self.state_manager.lock().await;
            _state_manager.flush_delta();
        }

        // Set the external ops counter to zero.
        self.external_ops_counter = 0;

        // Clear the passed calls.
        self.passed_calls.clear();
    }

    /// Returns the passed calls length.
    pub fn passed_calls_len(&self) -> usize {
        self.passed_calls.len()
    }

    /// Returns the passed calls.
    pub fn passed_calls(&self) -> Vec<(Call, OpsSpent, FeesSpent)> {
        self.passed_calls.clone()
    }

    /// Returns the external ops counter.
    pub fn external_ops_counter(&self) -> u32 {
        self.external_ops_counter
    }
}

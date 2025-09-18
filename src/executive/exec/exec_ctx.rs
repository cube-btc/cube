use crate::{
    constructive::entry::combinator::combinators::call::call::Call,
    executive::{
        exec::{caller::Caller, exec::execute, exec_error::ExecutionError},
        stack::stack_item::StackItem,
    },
    inscriptive::{
        coin_holder::coin_holder::COIN_HOLDER, repo::repo::PROGRAMS_REPO,
        state_holder::state_holder::STATE_HOLDER,
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
    state_holder: STATE_HOLDER,
    // The coin holder.
    coin_holder: COIN_HOLDER,
    // The programs repo.
    programs_repo: PROGRAMS_REPO,
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
        state_holder: &STATE_HOLDER,
        coin_holder: &COIN_HOLDER,
        programs_repo: &PROGRAMS_REPO,
        base_ops_price: u32,
        timestamp: u64,
    ) -> Self {
        Self {
            state_holder: Arc::clone(state_holder),
            coin_holder: Arc::clone(coin_holder),
            programs_repo: Arc::clone(programs_repo),
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
        let caller = Caller::new_account(call.account_key());

        // The contract id is the contract id of the called contract.
        let contract_id = call.contract_id();

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

        // State holder.
        let state_holder = &self.state_holder;

        // Pre-execution state backup.
        {
            let mut _state_holder = state_holder.lock().await;
            _state_holder.pre_execution();
        }

        let coin_holder = &self.coin_holder;

        // Pre-execution coin holder backup.
        {
            let mut _coin_holder = coin_holder.lock().await;
            _coin_holder.pre_execution();
        }

        // Programs repo.
        let programs_repo = &self.programs_repo;

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
            state_holder,
            coin_holder,
            programs_repo,
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
                // Rollback last on state holder.
                {
                    let mut _state_holder = state_holder.lock().await;
                    _state_holder.rollback_last();
                }

                // Rollback last on coin holder.
                {
                    let mut _coin_holder = coin_holder.lock().await;
                    _coin_holder.rollback_last();
                }

                // Return the error.
                return Err(error);
            }
        }
    }

    /// Flushes all the passed calls.
    pub async fn flush_all(&mut self) {
        // Rollback the state.
        {
            let mut _state_holder = self.state_holder.lock().await;
            _state_holder.rollback_all();
        }

        // Rollback the coin holder.
        {
            let mut _coin_holder = self.coin_holder.lock().await;
            _coin_holder.flush_delta();
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

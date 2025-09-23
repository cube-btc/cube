use crate::{
    executive::stack::{
        stack_error::{CoinBalanceGetError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{StackItemUintExt, StackUint},
    },
    inscriptive::coin_manager::coin_manager::COIN_MANAGER,
};

/// Pushes the BTC balance of the underlying contract into the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SELF_BALANCE;

impl OP_SELF_BALANCE {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        coin_manager: &COIN_MANAGER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the self contract id.
        let self_contract_id_bytes = stack_holder.contract_id();

        // Get the contract balance.
        let contract_balance = {
            let _coin_manager = coin_manager.lock().await;
            _coin_manager
                .get_contract_balance(self_contract_id_bytes)
                .ok_or(StackError::CoinBalanceGetError(
                    CoinBalanceGetError::UnableToGetContractBalance(self_contract_id_bytes),
                ))?
        };

        // Convert the contract balance to a stack uint.
        let contract_balance_as_stack_uint = StackUint::from(contract_balance);

        // Convert the contract balance to a stack item.
        let contract_balance_as_stack_item =
            StackItem::from_stack_uint(contract_balance_as_stack_uint);

        // Push the contract balance to the stack.
        stack_holder.push(contract_balance_as_stack_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SELF_BALANCE` opcode (0xc1).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc1]
    }
}

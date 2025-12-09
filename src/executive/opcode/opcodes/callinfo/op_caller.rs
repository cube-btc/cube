use crate::executive::{
    exec::caller::Caller,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes the caller type and id to the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CALLER;

/// The number of ops for the `OP_CALLER` opcode.
pub const CALLER_OPS: u32 = 1;

impl OP_CALLER {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Match caller.
        match stack_holder.caller() {
            Caller::Account(account_key) => {
                // Push the account key to the stack.
                stack_holder.push(StackItem::new(account_key.to_vec()))?;

                // Push false to the stack.
                // Represents caller type is an account.
                stack_holder.push(StackItem::false_item())?;
            }
            Caller::Contract(contract_id) => {
                // Push the contract id to the stack.
                stack_holder.push(StackItem::new(contract_id.to_vec()))?;

                // Push true to the stack.
                // Represents caller type is a contract.
                stack_holder.push(StackItem::true_item())?;
            }
        }

        // Increment the ops counter.
        stack_holder.increment_ops(CALLER_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_CALLER` opcode (0xb9).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb9]
    }
}

use crate::executive::{
    opcode::ops::OP_2DUP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Duplicates the top two stack items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2DUP;

impl OP_2DUP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the last stack item from the main stack.
        let last_item = stack_holder.last_item()?;

        // Clone the second-to-last stack item from the main stack.
        let second_to_last_item = stack_holder.item_by_depth(1)?;

        // Push the second-to-last stack item to the stack.
        stack_holder.push(second_to_last_item)?;

        // And then push the last stack item to the stack.
        stack_holder.push(last_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2DUP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2DUP` opcode (0x6e).
    pub fn bytecode() -> Vec<u8> {
        vec![0x6e]
    }
}

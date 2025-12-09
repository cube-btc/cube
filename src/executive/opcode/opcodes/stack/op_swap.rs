use crate::executive::{
    opcode::ops::OP_SWAP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// The top two items on the stack are swapped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SWAP;

impl OP_SWAP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the second-to-top stack item.
        let second_to_top_item = stack_holder.item_by_depth(1)?;

        // Remove the second-to-top stack item.
        stack_holder.remove_item_by_depth(1)?;

        // Push the item to the stack.
        stack_holder.push(second_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_SWAP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SWAP` opcode (0x7c).
    pub fn bytecode() -> Vec<u8> {
        vec![0x7c]
    }
}

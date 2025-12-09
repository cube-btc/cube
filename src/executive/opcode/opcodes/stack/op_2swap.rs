use crate::executive::{
    opcode::ops::OP_2SWAP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Swaps the top two pairs of items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2SWAP;

impl OP_2SWAP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the third-to-top stack item.
        let third_to_top_item = stack_holder.item_by_depth(2)?;

        // Clone the fourth-to-top stack item.
        let fourth_to_top_item = stack_holder.item_by_depth(3)?;

        // Remove the third-to-top stack item.
        stack_holder.remove_item_by_depth(2)?;

        // Remove the fourth-to-top stack item (removal depth is again 2).
        stack_holder.remove_item_by_depth(2)?;

        // Push the fourth-to-top stack item to the stack.
        stack_holder.push(fourth_to_top_item)?;

        // Push the third-to-top stack item to the stack.
        stack_holder.push(third_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2SWAP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2SWAP` opcode (0x72).
    pub fn bytecode() -> Vec<u8> {
        vec![0x72]
    }
}

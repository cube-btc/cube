use crate::executive::{
    opcode::ops::OP_TUCK_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// The item at the top of the stack is copied and inserted before the second-to-top item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_TUCK;

impl OP_TUCK {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the last stack item.
        let last_item = stack_holder.last_item()?;

        // Clone the second-to-top stack item.
        let second_to_top_item = stack_holder.item_by_depth(1)?;

        // Remove second to top item from the stack.
        stack_holder.remove_item_by_depth(1)?;

        // Push the second-to-top stack item to the stack.
        stack_holder.push(second_to_top_item)?;

        // Push the last stack item again to the stack.
        stack_holder.push(last_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_TUCK_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_TUCK` opcode (0x7d).
    pub fn bytecode() -> Vec<u8> {
        vec![0x7d]
    }
}

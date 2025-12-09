use crate::executive::{
    opcode::ops::OP_OVER_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Copies the second-to-top stack item to the top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_OVER;

impl OP_OVER {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the second-to-top stack item.
        let second_to_top_item = stack_holder.item_by_depth(1)?;

        // Push the item to the stack.
        stack_holder.push(second_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_OVER_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_OVER` opcode (0x78).
    pub fn bytecode() -> Vec<u8> {
        vec![0x78]
    }
}

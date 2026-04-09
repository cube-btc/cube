use crate::executive::{
    opcode::ops::OP_IFDUP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Duplicates the last item on the main stack if it is not zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_IFDUP;

impl OP_IFDUP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the last stack item.
        let last_item = stack_holder.last_item()?;

        // If the last item is not zero, duplicate it.
        if !last_item.is_false() {
            // Clone the last stack item from the main stack.
            let last_item = stack_holder.last_item()?;

            // Push the cloned value back to the main stack.
            stack_holder.push(last_item)?;
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_IFDUP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_IFDUP` opcode (0x73).
    pub fn bytecode() -> Vec<u8> {
        vec![0x73]
    }
}

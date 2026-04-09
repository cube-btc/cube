use crate::executive::{
    opcode::ops::OP_DUP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Duplicates the last item on the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_DUP;

impl OP_DUP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the last stack item from the main stack.
        let last_item = stack_holder.last_item()?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_DUP_OPS)?;

        // Push the cloned value back to the main stack.
        stack_holder.push(last_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_DUP` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x76]
    }
}

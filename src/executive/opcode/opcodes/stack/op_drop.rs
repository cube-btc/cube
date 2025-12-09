use crate::executive::{
    opcode::ops::OP_DROP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Drops the last item from the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_DROP;

impl OP_DROP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the last stack item from the main stack.
        stack_holder.pop()?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_DROP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_FROMALTSTACK` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x75]
    }
}

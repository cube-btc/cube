use crate::executive::{
    opcode::ops::OP_2DROP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Removes the top two stack items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2DROP;

impl OP_2DROP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop two items from the main stack.
        stack_holder.pop()?;
        stack_holder.pop()?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2DROP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2DROP` opcode (0x6d).
    pub fn bytecode() -> Vec<u8> {
        vec![0x6d]
    }
}

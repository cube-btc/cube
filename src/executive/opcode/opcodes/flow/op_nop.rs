use crate::executive::{
    opcode::ops::OP_NOP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// The `OP_NOP` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_NOP;

impl OP_NOP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }
        // Increment the ops counter.
        stack_holder.increment_ops(OP_NOP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_NOP` opcode (0x61).
    pub fn bytecode() -> Vec<u8> {
        vec![0x61]
    }
}

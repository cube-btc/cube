use crate::executive::{
    opcode::ops::OP_FAIL_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Fails the execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_FAIL;

impl OP_FAIL {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_FAIL_OPS)?;

        // Fail the execution.
        Err(StackError::FailError)
    }

    /// Returns the bytecode for the `OP_FAIL` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x6a]
    }
}

use crate::executive::{
    opcode::ops::OP_NIP_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// Removes the second-to-top stack item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_NIP;

impl OP_NIP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Remove the second-to-top stack item.
        stack_holder.remove_item_by_depth(1)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_NIP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_NIP` opcode (0x77).
    pub fn bytecode() -> Vec<u8> {
        vec![0x77]
    }
}

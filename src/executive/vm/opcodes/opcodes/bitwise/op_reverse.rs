use crate::executive::{
    opcode::ops::OP_REVERSE_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Reverses the byte order of the popped stack item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_REVERSE;

impl OP_REVERSE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop item from the main stack.
        let item = stack_holder.pop()?;

        // Get the bytes of the item.
        let item_bytes = item.bytes();

        // Reverse the bytes of the item.
        let reversed_bytes = item_bytes.iter().rev().copied().collect::<Vec<u8>>();

        // Push the reversed bytes to the main stack.
        stack_holder.push(StackItem::new(reversed_bytes))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_REVERSE_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_REVERSE` opcode (0x89).
    pub fn bytecode() -> Vec<u8> {
        vec![0x89]
    }
}

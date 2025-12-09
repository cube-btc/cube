use crate::executive::opcode::ops::OP_MFREE_OPS;
use crate::executive::stack::{
    limits::{MAX_KEY_LENGTH, MIN_KEY_LENGTH},
    stack_error::{MemoryError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use serde::{Deserialize, Serialize};

/// The `OP_MFREE` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_MFREE;

impl OP_MFREE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop key
        let key = stack_holder.pop()?;

        // Make sure key is within the valid length range (1 to 40 bytes).
        if key.len() < MIN_KEY_LENGTH || key.len() > MAX_KEY_LENGTH {
            return Err(StackError::MemoryError(
                MemoryError::InvalidMemoryKeyLength(key.len() as u8),
            ));
        }

        // Get contract memory.
        let memory = stack_holder.memory_mut();

        // Write to memory.
        let sweep_result_item = match memory.remove(&key.bytes().to_vec()) {
            // If the key already exists, push true value.
            Some(_) => StackItem::new(vec![0x01]),
            // If the key does not exist, push false value (empty vector).
            None => StackItem::new(vec![]),
        };

        // Increment the ops counter.
        stack_holder.increment_ops(OP_MFREE_OPS)?;

        // Push result to stack.
        stack_holder.push(sweep_result_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_MFREE` opcode (0xcc).
    pub fn bytecode() -> Vec<u8> {
        vec![0xcc]
    }
}

use crate::executive::{
    opcode::ops::OP_MREAD_OPS,
    stack::{
        limits::{MAX_KEY_LENGTH, MIN_KEY_LENGTH},
        stack_error::{MemoryError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
    },
};

/// The `OP_MREAD` opcode.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_MREAD;

impl OP_MREAD {
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
        let memory = stack_holder.memory();

        // Read from memory.
        let value = match memory.get(&key.bytes().to_vec()) {
            // If the value exists, push value.
            Some(value) => StackItem::new(value.clone()),
            // If the value does not exist, push false value (empty vector).
            None => StackItem::new(vec![]),
        };

        // Increment the ops counter.
        stack_holder.increment_ops(OP_MREAD_OPS)?;

        // Push result to stack.
        stack_holder.push(value)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_MREAD` opcode (0xcb).
    pub fn bytecode() -> Vec<u8> {
        vec![0xcb]
    }
}

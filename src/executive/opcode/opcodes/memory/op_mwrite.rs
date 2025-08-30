use crate::executive::{
    opcode::ops::OP_MWRITE_OPS,
    stack::{
        limits::{MAX_CONTRACT_MEMORY_SIZE, MAX_KEY_LENGTH, MIN_KEY_LENGTH, MIN_VALUE_LENGTH},
        stack_error::{MemoryError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
    },
};

/// The `OP_MWRITE` opcode.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_MWRITE;

impl OP_MWRITE {
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

        // Pop value
        let value = stack_holder.pop()?;

        // Make sure value is within the valid length range (1 to 4095 bytes).
        // NOTE: The maximum length of the value is bound by the stack item size limit.
        if value.len() < MIN_VALUE_LENGTH {
            return Err(StackError::MemoryError(
                MemoryError::InvalidMemoryValueLength(value.len() as u8),
            ));
        }

        // Get the contract's memory size.
        let contract_memory_size = stack_holder.memory_size();

        // New memory size.
        let new_contract_memory_size = match contract_memory_size + key.len() + value.len() {
            new_size if new_size < MAX_CONTRACT_MEMORY_SIZE => new_size,
            _ => {
                return Err(StackError::MemoryError(
                    MemoryError::ContractMemorySizeLimitExceeded,
                ));
            }
        };

        // Get contract memory.
        let memory = stack_holder.memory_mut();

        // Write to memory.
        let insertion_result_item =
            match memory.insert(key.bytes().to_vec(), value.bytes().to_vec()) {
                // If the key already exists, push true value.
                Some(_) => StackItem::new(vec![0x01]),
                // If the key does not exist, push false value (empty vector).
                None => StackItem::new(vec![]),
            };

        // Increment the ops counter.
        stack_holder.increment_ops(OP_MWRITE_OPS)?;

        // Update the contract's memory size.
        stack_holder.update_memory_size(new_contract_memory_size);

        // Push result to stack.
        stack_holder.push(insertion_result_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_MWRITE` opcode (0xca).
    pub fn bytecode() -> Vec<u8> {
        vec![0xca]
    }
}

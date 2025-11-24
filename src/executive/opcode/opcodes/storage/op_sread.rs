use crate::{
    executive::stack::{
        limits::{MAX_KEY_LENGTH, MIN_KEY_LENGTH},
        stack_error::{StackError, StorageError},
        stack_holder::StackHolder,
        stack_item::StackItem,
    },
    inscriptive::state_manager::state_manager::STATE_MANAGER,
};

/// The `OP_SREAD` opcode.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SREAD;

/// The number of ops for the `OP_SREAD` opcode.
pub const SREAD_OPS: u32 = 50;

impl OP_SREAD {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        state_manager: &STATE_MANAGER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop key
        let key = stack_holder.pop()?;

        // Make sure key is within the valid length range (1 to 40 bytes).
        if key.len() < MIN_KEY_LENGTH || key.len() > MAX_KEY_LENGTH {
            return Err(StackError::StorageError(
                StorageError::InvalidStorageKeyLength(key.len() as u8),
            ));
        }

        // Read from storage.
        let read_value = {
            let _state_manager = state_manager.lock().await;
            _state_manager.get_state_value(stack_holder.contract_id(), &key.bytes().to_vec())
        };

        // Push the read value to the main stack.
        match read_value {
            Some(read_value) => {
                stack_holder.push(StackItem::new(read_value))?;
            }
            None => {
                stack_holder.push(StackItem::false_item())?;
            }
        }

        // Increment the ops counter.
        stack_holder.increment_ops(SREAD_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SREAD` opcode (0xc9).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc9]
    }
}

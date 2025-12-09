use crate::executive::stack::{
    limits::{MAX_KEY_LENGTH, MIN_KEY_LENGTH, MIN_VALUE_LENGTH},
    stack_error::{StackError, StorageError},
    stack_holder::StackHolder,
};
use crate::inscriptive::state_manager::state_manager::STATE_MANAGER;
use serde::{Deserialize, Serialize};

/// The `OP_SWRITE` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SWRITE;

impl OP_SWRITE {
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

        // Pop value
        let value = stack_holder.pop()?;

        // Make sure value is within the valid length range (1 to 4095 bytes).
        // NOTE: The maximum length of the value is bound by the stack item size limit.
        if value.len() < MIN_VALUE_LENGTH {
            return Err(StackError::StorageError(
                StorageError::InvalidStorageValueLength(value.len() as u8),
            ));
        }

        // Write to storage.
        {
            let mut _state_manager = state_manager.lock().await;

            _state_manager
                .insert_update_state(
                    stack_holder.contract_id(),
                    &key.bytes().to_vec(),
                    &value.bytes().to_vec(),
                    true,
                )
                .map_err(|e| {
                    StackError::StorageError(StorageError::StateManagerInsertUpdateStateError(e))
                })?;
        }

        // Calculate the number of ops.
        let ops = calculate_ops(key.len() as u32, value.len() as u32);

        // Increment the ops counter.
        stack_holder.increment_ops(ops)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SWRITE` opcode (0xc8).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc8]
    }
}

const SWRITE_OPS_BASE: u32 = 50;
const SWRITE_OPS_MULTIPLIER: u32 = 1;

// Calculate the number of ops for a SWRITE opcode.
fn calculate_ops(key_length: u32, value_length: u32) -> u32 {
    let total_length = key_length + value_length;
    // Return the number of ops.
    SWRITE_OPS_BASE + (SWRITE_OPS_MULTIPLIER * total_length)
}

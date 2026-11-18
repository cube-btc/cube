use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use crate::{
    executive::stack::{
        limits::{MAX_KEY_LENGTH, MIN_KEY_LENGTH},
        stack_error::{StackError, StorageError},
        stack_holder::StackHolder,
        stack_item::StackItem,
    },
    inscriptive::state_manager::state_manager::STATE_MANAGER,
};
use serde::{Deserialize, Serialize};

/// The `OP_SFREE` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SFREE;

/// The number of ops for the `OP_SFREE` opcode.
impl OP_SFREE {
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

        // Free from storage.
        let sweep_result_item = {
            let mut _state_manager = state_manager.lock().await;

            let key_bytes = key.bytes().to_vec();
            let contract_id = stack_holder.contract_id();

            match _state_manager.get_state_value(contract_id, &key_bytes) {
                Some(_) => {
                    _state_manager
                        .remove_state(contract_id, &key_bytes, true)
                        .map_err(|e| {
                            StackError::StorageError(StorageError::StateManagerRemoveStateError(e))
                        })?;
                    StackItem::new(vec![0x01])
                }
                None => StackItem::new(vec![]),
            }
        };

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(stack_holder.opcode_ops().op_sfree))?;

        // Push result to stack.
        stack_holder.push(sweep_result_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SFREE` opcode (0xcf).
    pub fn bytecode() -> Vec<u8> {
        vec![0xcf]
    }
}

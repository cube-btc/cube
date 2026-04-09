use crate::{
    executive::stack::{
        stack_error::{ShadowOpsError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
    inscriptive::coin_manager::coin_manager::COIN_MANAGER,
};
use serde::{Deserialize, Serialize};

/// Returns the allocation value of an account within the contract shadow space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC_VAL;

impl OP_SHADOW_ALLOC_VAL {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        coin_manager: &COIN_MANAGER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the self contract id bytes.
        let self_contract_id_bytes = stack_holder.contract_id();

        // Pop the account key.
        let account_key = stack_holder.pop()?;

        // Convert the account key to bytes.
        let account_key_bytes: [u8; 32] = match account_key.bytes().try_into() {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(StackError::ShadowOpsError(
                    ShadowOpsError::InvalidAccountKeyBytes(account_key.bytes().to_vec()),
                ));
            }
        };

        // Check if the account key has an allocation within the contract shadow space by returning its allocation value.
        {
            // Get the mutable coin holder.
            let mut _coin_manager = coin_manager.lock().await;

            // Match the allocation value.
            match _coin_manager
                .get_shadow_alloc_value_in_satoshis(self_contract_id_bytes, account_key_bytes)
            {
                Some(value) => {
                    // Convert the value to a stack uint.
                    let value_as_stack_uint = StackUint::from_u64(value);

                    // Convert the value to a stack item.
                    let value_as_stack_item = StackItem::from_stack_uint(value_as_stack_uint);

                    // Push the value item to the main stack.
                    stack_holder.push(value_as_stack_item)?;

                    // Push true item to the main stack.
                    // Represents 'has allocation' = true.
                    stack_holder.push(StackItem::true_item())?;
                }
                None => {
                    // Push false item to the main stack.
                    // Represents 'has allocation' = false.
                    stack_holder.push(StackItem::false_item())?;
                }
            };
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC_VAL` opcode (0xc3).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc3]
    }
}

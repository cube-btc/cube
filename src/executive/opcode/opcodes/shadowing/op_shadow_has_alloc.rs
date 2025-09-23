use crate::{
    executive::stack::{
        stack_error::{ShadowOpsError, StackError},
        stack_holder::StackHolder,
        stack_item::StackItem,
    },
    inscriptive::coin_manager::coin_manager::COIN_MANAGER,
};

/// Checks if an account has an allocation within the contract shadow space.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_HAS_ALLOC;

impl OP_SHADOW_HAS_ALLOC {
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

        // Check if the account key has an allocation within the contract shadow space by returnin its allocation value.
        {
            // Get the mutable coin holder.
            let mut _coin_manager = coin_manager.lock().await;

            // Get the result item.
            let result_item = match _coin_manager
                .get_account_shadow_alloc_value_of_a_contract_in_sati_satoshis(
                    self_contract_id_bytes,
                    account_key_bytes,
                ) {
                Some(_) => StackItem::true_item(),
                None => StackItem::false_item(),
            };

            // Push the result item to the stack.
            stack_holder.push(result_item)?;
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_HAS_ALLOC` opcode (0xc2).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc2]
    }
}

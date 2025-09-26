use crate::{
    executive::stack::{
        stack_error::{ShadowOpsError, StackError},
        stack_holder::StackHolder,
    },
    inscriptive::coin_manager::coin_manager::COIN_MANAGER,
};

/// Allocates within the contract shadow space an account.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC;

impl OP_SHADOW_ALLOC {
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

        // Allocate the account key in the contract shadow space.
        {
            let mut _coin_manager = coin_manager.lock().await;
            _coin_manager
                .contract_shadow_alloc_account(self_contract_id_bytes, account_key_bytes)
                .map_err(|error| ShadowOpsError::ShadowAllocError(error))
                .map_err(StackError::ShadowOpsError)?;
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC` opcode (0xc0).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc0]
    }
}

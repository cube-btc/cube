use crate::{
    executive::stack::{
        stack_error::{ShadowOpsError, StackError},
        stack_holder::StackHolder,
    },
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Allocates within the contract shadow space an account.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC;

impl OP_SHADOW_ALLOC {
    pub async fn execute(
        stack_holder: &mut StackHolder,
        coin_holder: &COIN_HOLDER,
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

        // Get the contract coin holder.
        let contract_coin_holder = {
            let _coin_holder = coin_holder.lock().await;
            _coin_holder.contract_coin_holder()
        };

        // Allocate the account key in the contract shadow space.
        {
            let mut _contract_coin_holder = contract_coin_holder.lock().await;
            _contract_coin_holder
                .shadow_alloc(self_contract_id_bytes, account_key_bytes)
                .map_err(|error| ShadowOpsError::ShadowAllocError(error))
                .map_err(StackError::ShadowOpsError)?;
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC` opcode (0xc3).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc3]
    }
}

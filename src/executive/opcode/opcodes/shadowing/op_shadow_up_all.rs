use crate::executive::stack::{
    stack_error::{ShadowOpsError, StackError},
    stack_holder::StackHolder,
    stack_uint::{SafeConverter, StackItemUintExt},
};
use crate::inscriptive::coin_manager::coin_manager::COIN_MANAGER;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_UP_ALL;

impl OP_SHADOW_UP_ALL {
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

        // Pop the amount.
        let amount = stack_holder.pop()?;

        // Convert the amount to a stack uint.
        let amount_as_stack_uint = amount.to_stack_uint().ok_or(StackError::ShadowOpsError(
            ShadowOpsError::InvalidAmountBytes(amount.bytes().to_vec()),
        ))?;

        // Convert the amount to a u64.
        let amount_as_u64 = amount_as_stack_uint
            .to_u64()
            .ok_or(StackError::ShadowOpsError(
                ShadowOpsError::InvalidAmountBytes(amount.bytes().to_vec()),
            ))?;

        // Allocate the account key in the contract shadow space.
        {
            let mut _coin_manager = coin_manager.lock().await;
            _coin_manager
                .shadow_up_all(self_contract_id_bytes, amount_as_u64)
                .map_err(|error| ShadowOpsError::ShadowAllocUpAllError(error))
                .map_err(StackError::ShadowOpsError)?;
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_UP_ALL` opcode (0xc6).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc6]
    }
}

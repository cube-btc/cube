use crate::{
    executive::stack::{
        stack_error::StackError,
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
    inscriptive::coin_manager::coin_manager::COIN_MANAGER,
};

/// Returns the number of total shadow allocations of the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_NUM_ALLOCS;

impl OP_SHADOW_NUM_ALLOCS {
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

        //
        {
            // Get the mutable coin holder.
            let mut _coin_manager = coin_manager.lock().await;

            // Get the result item.
            let result_item = match _coin_manager.get_contract_num_allocs(self_contract_id_bytes) {
                Some(num_allocs) => {
                    // Convert the number of allocations to a stack uint.
                    let num_allocs_as_stack_uint = StackUint::from_u64(num_allocs);

                    // Return the result item.
                    StackItem::from_stack_uint(num_allocs_as_stack_uint)
                }
                // NOTE: This is impossible.
                None => StackItem::false_item(),
            };

            // Push the result item to the stack.
            stack_holder.push(result_item)?;
        }

        // Return the result.
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_NUM_ALLOCS` opcode (0xc8).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc8]
    }
}

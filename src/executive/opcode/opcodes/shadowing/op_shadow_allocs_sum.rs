use crate::{
    executive::stack::{
        stack_error::StackError,
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Returns the sum of all shadow allocation values of the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOCS_SUM;

impl OP_SHADOW_ALLOCS_SUM {
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

        // Get the contract coin holder.
        let contract_coin_holder = {
            let _coin_holder = coin_holder.lock().await;
            _coin_holder.contract_coin_holder()
        };

        //
        {
            // Get the mutable contract coin holder.
            let mut _contract_coin_holder = contract_coin_holder.lock().await;

            // Get the result item.
            let result_item =
                match _contract_coin_holder.get_contract_allocs_sum(self_contract_id_bytes) {
                    Some(allocs_sum) => {
                        // Convert the number of allocations to a stack uint.
                        let allocs_sum_as_stack_uint = StackUint::from_u64(allocs_sum);

                        // Return the result item.
                        StackItem::from_stack_uint(allocs_sum_as_stack_uint)
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

    /// Returns the bytecode for the `OP_SHADOW_ALLOCS_SUM` opcode (0xc9).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc9]
    }
}

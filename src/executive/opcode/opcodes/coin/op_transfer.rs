use crate::{
    executive::stack::{stack_error::StackError, stack_holder::StackHolder},
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Transfers coins from the contract into an account or to another contract.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_TRANSFER;

impl OP_TRANSFER {
    pub fn execute(
        stack_holder: &mut StackHolder,
        _coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        Ok(())
    }

    /// Returns the bytecode for the `OP_TRANSFER` opcode (0xc2).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc2]
    }
}

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
        _stack_holder: &mut StackHolder,
        _coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_TRANSFER` opcode (0xc1).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc1]
    }
}

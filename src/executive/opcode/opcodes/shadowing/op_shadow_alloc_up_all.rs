use crate::{
    executive::stack::{stack_error::StackError, stack_holder::StackHolder},
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Shadow allocation up all.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC_UP_ALL;

impl OP_SHADOW_ALLOC_UP_ALL {
    pub fn execute(
        _stack_holder: &mut StackHolder,
        _coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC_UP_ALL` opcode (0xc6).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc6]
    }
}

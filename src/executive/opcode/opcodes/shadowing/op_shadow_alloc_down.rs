use crate::{
    executive::stack::{stack_error::StackError, stack_holder::StackHolder},
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Shadow allocation down.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC_DOWN;

impl OP_SHADOW_ALLOC_DOWN {
    pub fn execute(
        _stack_holder: &mut StackHolder,
        _coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC_DOWN` opcode (0xc5).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc5]
    }
}

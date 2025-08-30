use crate::{
    executive::stack::{stack_error::StackError, stack_holder::StackHolder},
    inscriptive::coin_holder::coin_holder::COIN_HOLDER,
};

/// Shadow allocation up.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC_UP;

impl OP_SHADOW_ALLOC_UP {
    pub fn execute(
        _stack_holder: &mut StackHolder,
        _coin_holder: &COIN_HOLDER,
    ) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC_UP` opcode (0xc4).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc4]
    }
}

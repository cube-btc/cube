use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};

/// Shadow allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC;

impl OP_SHADOW_ALLOC {
    pub fn execute(_stack_holder: &mut StackHolder) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC` opcode (0xc2).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc2]
    }
}

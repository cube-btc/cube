use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};

/// Shadow allocation up.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_SHADOW_ALLOC_UP;

impl OP_SHADOW_ALLOC_UP {
    pub fn execute(_stack_holder: &mut StackHolder) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_SHADOW_ALLOC_UP` opcode (0xc3).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc3]
    }
}

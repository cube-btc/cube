use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};

/// Pushes the account's individual BTC balance into the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_BALANCE;

impl OP_BALANCE {
    pub fn execute(_stack_holder: &mut StackHolder) -> Result<(), StackError> {
        Ok(())
    }

    /// Returns the bytecode for the `OP_BALANCE` opcode (0xc0).
    pub fn bytecode() -> Vec<u8> {
        vec![0xc0]
    }
}

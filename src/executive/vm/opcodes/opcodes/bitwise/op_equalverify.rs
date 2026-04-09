use crate::executive::{
    opcode::ops::OP_EQUALVERIFY_OPS,
    stack::{
        stack_error::{MandatoryError, StackError},
        stack_holder::StackHolder,
    },
};
use serde::{Deserialize, Serialize};

/// Same as OP_EQUAL, but runs OP_VERIFY afterward.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_EQUALVERIFY;

impl OP_EQUALVERIFY {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop two items from the main stack.
        let item_1 = stack_holder.pop()?;
        let item_2 = stack_holder.pop()?;

        // Check if the two items are equal.
        if item_1.bytes() != item_2.bytes() {
            return Err(StackError::MandatoryError(
                MandatoryError::MandatoryEqualVerifyError,
            ));
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_EQUALVERIFY_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_EQUALVERIFY` opcode (0x88).
    pub fn bytecode() -> Vec<u8> {
        vec![0x88]
    }
}

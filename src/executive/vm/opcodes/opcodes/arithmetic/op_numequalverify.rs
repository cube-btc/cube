use crate::executive::{
    opcode::ops::OP_NUMEQUALVERIFY_OPS,
    stack::{
        stack_error::{MandatoryError, StackError, StackUintError},
        stack_holder::StackHolder,
        stack_uint::StackItemUintExt,
    },
};
use serde::{Deserialize, Serialize};

/// Same as OP_NUMEQUAL, but runs OP_VERIFY afterward.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_NUMEQUALVERIFY;

impl OP_NUMEQUALVERIFY {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the first item from the main stack.
        let item_1 = stack_holder.pop()?;

        // Pop the second item from the main stack.
        let item_2 = stack_holder.pop()?;

        // Convert item 1 to a stack uint.
        let num_1 = item_1.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Convert item 2 to a stack uint.
        let num_2 = item_2.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Return an error if the numbers are not equal.
        if num_1 != num_2 {
            return Err(StackError::MandatoryError(
                MandatoryError::MandatoryEqualVerifyError,
            ));
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_NUMEQUALVERIFY_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_NUMEQUALVERIFY` opcode (0x9d).
    pub fn bytecode() -> Vec<u8> {
        vec![0x9d]
    }
}

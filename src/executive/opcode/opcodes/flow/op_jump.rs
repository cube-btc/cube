use crate::executive::stack::{
    stack_error::{StackError, StackUintError},
    stack_holder::StackHolder,
    stack_uint::{SafeConverter, StackItemUintExt},
};

/// The number of ops for the `OP_JUMP` opcode.
const JUMP_OPS: u32 = 1;

/// Jumps to the respective opcode execution.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub struct OP_JUMP;

impl OP_JUMP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<usize, StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(0);
        }

        // Get the iterator index from the stack.
        let iterator_index_item = stack_holder.pop()?;

        // Pop the iterator index from the stack.
        let iterator_index_as_stack_uint =
            iterator_index_item
                .to_stack_uint()
                .ok_or(StackError::StackUintError(
                    StackUintError::StackUintConversionError,
                ))?;

        // Convert the iterator index to a u32.
        let iterator_index_as_usize =
            iterator_index_as_stack_uint
                .to_usize()
                .ok_or(StackError::StackUintError(
                    StackUintError::StackUintConversionError,
                ))?;

        // Increment the ops counter.
        stack_holder.increment_ops(JUMP_OPS)?;

        // Return the iterator index.
        Ok(iterator_index_as_usize)
    }

    /// Returns the bytecode for the `OP_JUMP` opcode (0x62).
    pub fn bytecode() -> Vec<u8> {
        vec![0x62]
    }
}

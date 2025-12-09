use crate::executive::{
    opcode::ops::OP_RIGHT_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt},
    },
};
use serde::{Deserialize, Serialize};

/// Keeps only characters right of the specified point in a string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_RIGHT;

impl OP_RIGHT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop split index from stack.
        let split_index_item = stack_holder.pop()?;

        // Pop byte array from stack.
        let byte_array_item = stack_holder.pop()?;

        // Convert split index to stack uint and then to usize.
        let split_index = split_index_item
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintMaxOverflowError,
            ))?
            .to_usize()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?;

        // Get the bytes from the StackItem
        let bytes_slice = byte_array_item.bytes();

        // Check if split index is valid
        if split_index > bytes_slice.len() {
            return Err(StackError::SplitIndexError);
        }

        // Split the byte array into two stack items at the index.
        let right_slice = {
            // Split at the index
            let (_, right_slice) = bytes_slice.split_at(split_index);

            // Return the two stack items.
            right_slice.to_vec()
        };

        // Push the right item back to the main stack.
        stack_holder.push(StackItem::new(right_slice))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_RIGHT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_RIGHT` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x81]
    }
}

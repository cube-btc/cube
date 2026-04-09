use crate::executive::{
    opcode::ops::OP_WITHIN_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::StackItemUintExt,
    },
};
use serde::{Deserialize, Serialize};

/// Returns 1 if x is within the specified range (left-inclusive), 0 otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_WITHIN;

impl OP_WITHIN {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the first item from the main stack.
        let max = stack_holder.pop()?;

        // Pop the second item from the main stack.
        let min = stack_holder.pop()?;

        // Pop the third item from the main stack.
        let num = stack_holder.pop()?;

        // Convert max to a stack uint.
        let max = max.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Convert min to a stack uint.
        let min = min.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Convert num to a stack uint.
        let num = num.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Push 1 if num is within the range, 0 otherwise.
        match num >= min && num <= max {
            true => stack_holder.push(StackItem::true_item())?,
            false => stack_holder.push(StackItem::false_item())?,
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_WITHIN_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_WITHIN` opcode (0xa5).
    pub fn bytecode() -> Vec<u8> {
        vec![0xa5]
    }
}

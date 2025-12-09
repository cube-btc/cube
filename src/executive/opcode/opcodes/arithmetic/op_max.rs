use crate::executive::{
    opcode::ops::OP_MAX_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::StackItemUintExt,
    },
};
use serde::{Deserialize, Serialize};

/// Returns the larger of a and b.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_MAX;

impl OP_MAX {
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

        // Get the larger of the two numbers.
        let larger = num_1.max(num_2);

        // Push the larger of the two numbers.
        stack_holder.push(StackItem::from_stack_uint(larger))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_MAX_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_MAX` opcode (0xa4).
    pub fn bytecode() -> Vec<u8> {
        vec![0xa4]
    }
}

use crate::executive::{
    opcode::ops::OP_1SUB_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// 1 is subtracted from the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_1SUB;

impl OP_1SUB {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the top item from the main stack.
        let item = stack_holder.pop()?;

        // Convert the item to a `StackUint`.
        let item_uint = item.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Subtract 1 from the item.
        match item_uint.checked_sub(StackUint::from_u64(1)) {
            // If the result is an overflow, return False (an empty stack item).
            None => {
                // Push old value to the main stack.
                stack_holder.push(item)?;

                // Push False (an empty stack item) to the main stack.
                stack_holder.push(StackItem::false_item())?;
            }
            // If the result is not an overflow, return the result.
            Some(result) => {
                // Push the result to the main stack.
                stack_holder.push(StackItem::from_stack_uint(result))?;

                // Push True to the main stack.
                stack_holder.push(StackItem::true_item())?;
            }
        };

        // Increment the ops counter.
        stack_holder.increment_ops(OP_1SUB_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_1SUB` opcode (0x8c).
    pub fn bytecode() -> Vec<u8> {
        vec![0x8c]
    }
}

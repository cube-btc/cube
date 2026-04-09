use crate::executive::{
    opcode::ops::OP_SUB_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::StackItemUintExt,
    },
};
use serde::{Deserialize, Serialize};

/// Subtracts two items on the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SUB;

impl OP_SUB {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop two items from the main stack.
        let item_1 = stack_holder.pop()?;
        let item_2 = stack_holder.pop()?;

        // Irem 1 uint value;
        let item_1_uint = item_1.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintMaxOverflowError,
        ))?;

        // Item 2 uint value;
        let item_2_uint = item_2.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintMaxOverflowError,
        ))?;

        // Subtract the two values.
        match item_1_uint.checked_sub(item_2_uint) {
            // If the result is an overflow, return False (an empty stack item).
            None => {
                // Push old value to the main stack.
                stack_holder.push(item_1)?;

                // Push old value to the main stack.
                stack_holder.push(item_2)?;

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
        stack_holder.increment_ops(OP_SUB_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SUB` opcode (0x94).
    pub fn bytecode() -> Vec<u8> {
        vec![0x94]
    }
}

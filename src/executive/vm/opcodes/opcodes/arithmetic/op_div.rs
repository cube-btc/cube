use crate::executive::{
    opcode::ops::OP_DIV_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// Divides two items on the main stack. Returns the modulo and division result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_DIV;

impl OP_DIV {
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
            StackUintError::StackUintConversionError,
        ))?;

        // Item 2 uint value;
        let item_2_uint = item_2.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintConversionError,
        ))?;

        // Check if the divisor is zero.
        match item_2_uint == StackUint::zero() {
            true => {
                // Push old value to the main stack.
                stack_holder.push(item_1)?;

                // Push old value to the main stack.
                stack_holder.push(item_2)?;

                // Push False (an empty stack item) to the main stack.
                stack_holder.push(StackItem::false_item())?;
            }
            false => {
                // Divide the two values.
                let (division, modulo) = item_1_uint.div_mod(item_2_uint);

                // Push the modulo result to the main stack.
                stack_holder.push(StackItem::from_stack_uint(modulo))?;

                // Push the division result to the main stack.
                stack_holder.push(StackItem::from_stack_uint(division))?;

                // Push true to the main stack.
                stack_holder.push(StackItem::true_item())?;
            }
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_DIV_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_DIV` opcode (0x96).
    pub fn bytecode() -> Vec<u8> {
        vec![0x96]
    }
}

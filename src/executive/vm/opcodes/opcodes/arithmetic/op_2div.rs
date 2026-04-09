use crate::executive::{
    opcode::ops::OP_2DIV_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// Divides the top item on the main stack by 2. Returns the modulo and division result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2DIV;

impl OP_2DIV {
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

        // Divide the item by 2.
        let (division, modulo) = item_uint.div_mod(StackUint::from_u64(2));

        // Push the modulo result to the main stack.
        stack_holder.push(StackItem::from_stack_uint(modulo))?;

        // Push the division result to the main stack.
        stack_holder.push(StackItem::from_stack_uint(division))?;

        // Push true to the main stack.
        // Unlike OP_DIV, OP_2DIV division result will never be false.
        stack_holder.push(StackItem::true_item())?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2DIV_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2DIV` opcode (0x8e).
    pub fn bytecode() -> Vec<u8> {
        vec![0x8e]
    }
}

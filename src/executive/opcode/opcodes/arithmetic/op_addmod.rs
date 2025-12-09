use crate::executive::{
    opcode::ops::OP_ADDMOD_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// Adds two items on the main stack and returns the result modulo MAX::U256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ADDMOD;

impl OP_ADDMOD {
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

        // Add the two values modulo MAX::U256.
        let result = StackItem::from_stack_uint(StackUint::addmod(&item_1_uint, &item_2_uint));

        // Increment the ops counter.
        stack_holder.increment_ops(OP_ADDMOD_OPS)?;

        // Push the result to the main stack.
        stack_holder.push(result)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ADDMOD` opcode (0x8f).
    pub fn bytecode() -> Vec<u8> {
        vec![0x8f]
    }
}

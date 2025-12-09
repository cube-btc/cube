use crate::executive::{
    opcode::ops::OP_ROLL_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_uint::StackItemUintExt,
    },
};
use serde::{Deserialize, Serialize};

/// Rolls an item from the main stack to the top of the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ROLL;

impl OP_ROLL {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the last item from stack.
        let last_item = stack_holder.pop()?;

        // Get the pick depth from the last item.
        let pick_depth = last_item.to_stack_uint().ok_or(StackError::StackUintError(
            StackUintError::StackUintMaxOverflowError,
        ))?;

        // Get the item at the pick depth.
        let item = stack_holder.item_by_depth(pick_depth.as_u32())?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_ROLL_OPS)?;

        // Remove the item at the pick depth.
        stack_holder.remove_item_by_depth(pick_depth.as_u32())?;

        // Push the item onto the stack.
        stack_holder.push(item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ROLL` opcode (0x7a).
    pub fn bytecode() -> Vec<u8> {
        vec![0x7a]
    }
}

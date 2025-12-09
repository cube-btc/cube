use crate::executive::{
    opcode::ops::OP_2ROT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder},
};
use serde::{Deserialize, Serialize};

/// The fifth and sixth items back are moved to the top of the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2ROT;

impl OP_2ROT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the fifth-to-top stack item.
        let fifth_to_top_item = stack_holder.item_by_depth(4)?;

        // Clone the sixth-to-top stack item.
        let sixth_to_top_item = stack_holder.item_by_depth(5)?;

        // Remove the fifth-to-top stack item.
        stack_holder.remove_item_by_depth(4)?;

        // Remove the sixth-to-top stack item (removal depth is again 4).
        stack_holder.remove_item_by_depth(4)?;

        // Push the sixth-to-top stack item to the stack.
        stack_holder.push(sixth_to_top_item)?;

        // Push the fifth-to-top stack item to the stack.
        stack_holder.push(fifth_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2ROT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2ROT` opcode (0x71).
    pub fn bytecode() -> Vec<u8> {
        vec![0x71]
    }
}

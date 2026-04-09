use crate::executive::{
    opcode::ops::OP_DEPTH_OPS,
    stack::{
        stack_error::StackError,
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// Puts the number of stack items onto the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_DEPTH;

impl OP_DEPTH {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the depth of the main stack.
        let depth = stack_holder.stack_items_count();

        // Convert the depth to a stack uint.
        let depth_as_stack_uint = StackUint::from_u32(depth);

        // Convert the stack uint to a stack item.
        let depth_as_stack_item = StackItem::from_stack_uint(depth_as_stack_uint);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_DEPTH_OPS)?;

        // Push the depth to the main stack.
        stack_holder.push(depth_as_stack_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_DEPTH` opcode (0x74).
    pub fn bytecode() -> Vec<u8> {
        vec![0x74]
    }
}

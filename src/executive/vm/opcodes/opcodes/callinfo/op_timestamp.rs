use crate::executive::stack::{
    stack_error::StackError,
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use serde::{Deserialize, Serialize};

/// Push the timestamp to the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_TIMESTAMP;

/// The number of ops for the `OP_TIMESTAMP` opcode.
pub const TIMESTAMP_OPS: u32 = 1;

impl OP_TIMESTAMP {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the timestamp as a u64.
        let timestamp_as_u64 = stack_holder.timestamp();

        // Convert the timestamp to a stack int.
        let timestamp_as_stack_uint = StackUint::from_u64(timestamp_as_u64);

        // Convert the stack int to stack item.
        let timestamp_as_stack_item = StackItem::from_stack_uint(timestamp_as_stack_uint);

        // Push the item to the main stack.
        stack_holder.push(timestamp_as_stack_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(TIMESTAMP_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_TIMESTAMP` opcode (0xbd).
    pub fn bytecode() -> Vec<u8> {
        vec![0xbd]
    }
}

use crate::executive::{
    opcode::ops::OP_SIZE_OPS,
    stack::{
        stack_error::StackError,
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt, StackUint},
    },
};
use serde::{Deserialize, Serialize};

/// Pushes the string length of the top element of the stack (without popping it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SIZE;

impl OP_SIZE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the last item from the stack.
        let item = stack_holder.last_item()?;

        // Get the size of the item.
        let item_size = item.bytes().len() as u32;

        // Convert the size to a stack uint.
        let item_size_stack_uint = StackUint::from_u32(item_size);

        // Convert the stack uint to a stack item.
        let item_size_stack_item = StackItem::from_stack_uint(item_size_stack_uint);

        // Push the size of the item back to the stack.
        stack_holder.push(item_size_stack_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_SIZE_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SIZE` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x82]
    }
}

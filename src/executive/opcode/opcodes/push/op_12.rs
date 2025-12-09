use crate::executive::{
    opcode::ops::OP_12_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 12 (0x0c) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_12;

impl OP_12 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 12 (0x0c) to the main stack.
        let item_to_push = StackItem::new(vec![0x0c]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_12_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_12` opcode (0x5c).
    pub fn bytecode() -> Vec<u8> {
        vec![0x5c]
    }
}

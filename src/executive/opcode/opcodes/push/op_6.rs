use crate::executive::{
    opcode::ops::OP_6_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 6 (0x06) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_6;

impl OP_6 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 6 (0x06) to the main stack.
        let item_to_push = StackItem::new(vec![0x06]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_6_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_6` opcode (0x56).
    pub fn bytecode() -> Vec<u8> {
        vec![0x56]
    }
}

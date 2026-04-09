use crate::executive::{
    opcode::ops::OP_15_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 15 (0x0f) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_15;

impl OP_15 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 15 (0x0f) to the main stack.
        let item_to_push = StackItem::new(vec![0x0f]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_15_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_15` opcode (0x5f).
    pub fn bytecode() -> Vec<u8> {
        vec![0x5f]
    }
}

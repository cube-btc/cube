use crate::executive::{
    opcode::ops::OP_3_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 3 (0x03) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_3;

impl OP_3 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 3 (0x03) to the main stack.
        let item_to_push = StackItem::new(vec![0x03]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_3_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_3` opcode (0x53).
    pub fn bytecode() -> Vec<u8> {
        vec![0x53]
    }
}

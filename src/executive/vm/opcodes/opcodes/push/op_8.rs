use crate::executive::{
    opcode::ops::OP_8_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 8 (0x08) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_8;

impl OP_8 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 8 (0x08) to the main stack.
        let item_to_push = StackItem::new(vec![0x08]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_8_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_8` opcode (0x58).
    pub fn bytecode() -> Vec<u8> {
        vec![0x58]
    }
}

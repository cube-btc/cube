use crate::executive::{
    opcode::ops::OP_FALSE_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 0, also known as false (empty byte array) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_FALSE;

impl OP_FALSE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push False (empty stack item) to the main stack.
        let item_to_push = StackItem::new(vec![]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_FALSE_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_FALSE` opcode (0x00).
    pub fn bytecode() -> Vec<u8> {
        vec![0x00]
    }
}

/// OP_0 is analogous to `OP_FALSE`.
#[allow(non_camel_case_types)]
pub type OP_0 = OP_FALSE;

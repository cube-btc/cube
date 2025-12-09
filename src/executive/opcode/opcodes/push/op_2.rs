use crate::executive::{
    opcode::ops::OP_2_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 2 (0x02) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2;

impl OP_2 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push 2 (0x02) to the main stack.
        let item_to_push = StackItem::new(vec![0x02]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_2_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2` opcode (0x52).
    pub fn bytecode() -> Vec<u8> {
        vec![0x52]
    }
}

use crate::executive::{
    opcode::ops::OP_TRUE_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Pushes number 1, also known as true (0x01) to the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_TRUE;

impl OP_TRUE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Push True (0x01) to the main stack.
        let item_to_push = StackItem::new(vec![0x01]);

        // Increment the ops counter.
        stack_holder.increment_ops(OP_TRUE_OPS)?;

        // Push the item to the main stack.
        stack_holder.push(item_to_push)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_TRUE` opcode (0x51).
    pub fn bytecode() -> Vec<u8> {
        vec![0x51]
    }
}

/// OP_1 is analogous to `OP_TRUE`.
#[allow(non_camel_case_types)]
pub type OP_1 = OP_TRUE;

use crate::executive::{
    opcode::ops::OP_INVERT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Flips all of the bits in the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_INVERT;

impl OP_INVERT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop one item from the main stack.
        let item = stack_holder.pop()?;

        // Invert the bits of the item.
        let inverted_item = item.bytes().iter().map(|b| !b).collect::<Vec<u8>>();

        // Push the item to the main stack.
        stack_holder.push(StackItem::new(inverted_item))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_INVERT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_INVERT` opcode (0x83).
    pub fn bytecode() -> Vec<u8> {
        vec![0x83]
    }
}

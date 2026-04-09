use crate::executive::{
    opcode::ops::OP_0NOTEQUAL_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Returns 0 if the input is 0. 1 otherwise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_0NOTEQUAL;

impl OP_0NOTEQUAL {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the top item from the main stack.
        let item = stack_holder.pop()?;

        // Check if the item is false, and push false if it is, otherwise true.
        match item.is_false() {
            true => stack_holder.push(StackItem::false_item())?,
            false => stack_holder.push(StackItem::true_item())?,
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_0NOTEQUAL_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_0NOTEQUAL` opcode (0x92).
    pub fn bytecode() -> Vec<u8> {
        vec![0x92]
    }
}

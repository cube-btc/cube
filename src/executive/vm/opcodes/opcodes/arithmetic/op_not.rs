use crate::executive::{
    opcode::ops::OP_NOT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// If the input is 0 or 1, it is flipped. Otherwise the output will be 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_NOT;

impl OP_NOT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the top item from the main stack.
        let item = stack_holder.pop()?;

        // Check if the item is true, and push the opposite item to the main stack.
        match item.is_true() {
            true => stack_holder.push(StackItem::false_item())?,
            false => stack_holder.push(StackItem::true_item())?,
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_NOT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_NOT` opcode (0x91).
    pub fn bytecode() -> Vec<u8> {
        vec![0x91]
    }
}

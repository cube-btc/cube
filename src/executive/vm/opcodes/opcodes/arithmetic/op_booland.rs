use crate::executive::{
    opcode::ops::OP_BOOLAND_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// If both a and b are not 0, the output is 1. Otherwise 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_BOOLAND;

impl OP_BOOLAND {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the first item from the main stack.
        let item_1 = stack_holder.pop()?;

        // Pop the second item from the main stack.
        let item_2 = stack_holder.pop()?;

        // If both items are not false, push true, otherwise push false.
        if !item_1.is_false() && !item_2.is_false() {
            stack_holder.push(StackItem::true_item())?;
        } else {
            stack_holder.push(StackItem::false_item())?;
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_BOOLAND_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_BOOLAND` opcode (0x9a).
    pub fn bytecode() -> Vec<u8> {
        vec![0x9a]
    }
}

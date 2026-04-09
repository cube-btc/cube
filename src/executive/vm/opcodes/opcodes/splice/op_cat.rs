use crate::executive::{
    opcode::ops::OP_CAT_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Concatenates two items on the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CAT;

impl OP_CAT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop item one from the main stack.
        let item_1 = stack_holder.pop()?;

        // Pop item two from the main stack.
        let item_2 = stack_holder.pop()?;

        // Join the two items
        let mut joined = Vec::<u8>::with_capacity(item_1.len() as usize + item_2.len() as usize);
        joined.extend(item_2.bytes());
        joined.extend(item_1.bytes());

        // Push the joined item back to the main stack.
        stack_holder.push(StackItem::new(joined))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_CAT_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_CAT` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x7e]
    }
}

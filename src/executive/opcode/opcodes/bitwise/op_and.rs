use crate::executive::{
    opcode::ops::OP_AND_OPS,
    stack::{stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem},
};
use serde::{Deserialize, Serialize};

/// Boolean and between each bit in the inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_AND;

impl OP_AND {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop first item from the main stack.
        let item_1 = stack_holder.pop()?;

        // Pop second item from the main stack.
        let item_2 = stack_holder.pop()?;

        // Boolean and between each bit in the inputs.
        let and_item = item_1
            .bytes()
            .iter()
            .zip(item_2.bytes().iter())
            .map(|(b1, b2)| b1 & b2)
            .collect::<Vec<u8>>();

        // Push the item to the main stack.
        stack_holder.push(StackItem::new(and_item))?;

        // Increment the ops counter.
        stack_holder.increment_ops(OP_AND_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_AND` opcode (0x84).
    pub fn bytecode() -> Vec<u8> {
        vec![0x84]
    }
}

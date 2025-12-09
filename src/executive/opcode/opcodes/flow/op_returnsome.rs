use crate::executive::{
    opcode::ops::OP_RETURNSOME_OPS,
    stack::{
        stack_error::{StackError, StackUintError},
        stack_holder::StackHolder,
        stack_item::StackItem,
        stack_uint::{SafeConverter, StackItemUintExt},
    },
};
use serde::{Deserialize, Serialize};

/// Returns some number of items from the main stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_RETURNSOME;

impl OP_RETURNSOME {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<Vec<StackItem>, StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(vec![]);
        }

        // Pop the number of items to return from the stack.
        let items_count = stack_holder
            .pop()?
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?
            .to_usize()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?;

        // Collect remaining stack items.
        let mut items = Vec::<StackItem>::with_capacity(items_count);

        // Collect remaining stack items.
        for _ in 0..items_count {
            items.push(stack_holder.pop()?);
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_RETURNSOME_OPS)?;

        Ok(items)
    }

    /// Returns the bytecode for the `OP_RETURNSOME` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x66]
    }
}

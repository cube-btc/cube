use crate::executive::stack::{
    stack_error::{SecpError, StackError, StackUintError},
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use serde::{Deserialize, Serialize};

/// Adds two secp scalars together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SECPSCALARADD;

/// The number of ops for the `OP_SECPSCALARADD` opcode.
pub const SECPSCALARADD_OPS: u32 = 3;

impl OP_SECPSCALARADD {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop scalar 1 from the main stack.
        let scalar_1_item = stack_holder.pop()?;

        // Pop scalar 2 from the main stack.
        let scalar_2_item = stack_holder.pop()?;

        // Convert the first scalar to a secp scalar.
        let scalar_1 = scalar_1_item
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?
            .to_secp_scalar()
            .ok_or(StackError::SecpError(SecpError::InvalidSecpScalar))?;

        // Convert the second scalar to a secp scalar.
        let scalar_2 = scalar_2_item
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?
            .to_secp_scalar()
            .ok_or(StackError::SecpError(SecpError::InvalidSecpScalar))?;

        // Add the two scalars together.
        let addition = scalar_1 + scalar_2;

        // Convert the addition to a stack uint.
        let addition_stack_uint = StackUint::from_secp_scalar(addition);

        // Convert the stack uint to the stack item.
        let addition_item = StackItem::from_stack_uint(addition_stack_uint);

        // Push the addition back to the main stack.
        stack_holder.push(addition_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(SECPSCALARADD_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SECPSCALARADD` opcode (0xae).
    pub fn bytecode() -> Vec<u8> {
        vec![0xae]
    }
}

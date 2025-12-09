use crate::executive::stack::{
    stack_error::{SecpError, StackError, StackUintError},
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use serde::{Deserialize, Serialize};

/// Multiplies two secp scalars together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SECPSCALARMUL;

/// The number of ops for the `OP_SECPSCALARMUL` opcode.
pub const SECPSCALARMUL_OPS: u32 = 5;

impl OP_SECPSCALARMUL {
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

        // Multiply the two scalars together.
        let multiplication = scalar_1 * scalar_2;

        // Convert the multiplication to a stack uint.
        let multiplication_stack_uint = StackUint::from_secp_scalar(multiplication);

        // Convert the stack uint to the stack item.
        let multiplication_item = StackItem::from_stack_uint(multiplication_stack_uint);

        // Push the multiplication back to the main stack.
        stack_holder.push(multiplication_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(SECPSCALARMUL_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SECPSCALARMUL` opcode (0xaf).
    pub fn bytecode() -> Vec<u8> {
        vec![0xaf]
    }
}

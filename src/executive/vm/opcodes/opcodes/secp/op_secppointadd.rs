use crate::executive::stack::{
    stack_error::{SecpError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use secp::MaybePoint;
use serde::{Deserialize, Serialize};

/// Adds two secp points together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SECPPOINTADD;

/// The number of ops for the `OP_SECPPOINTADD` opcode.
pub const SECPSPOINTADD_OPS: u32 = 10;

impl OP_SECPPOINTADD {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop point 1 from the main stack.
        let point_1_item = stack_holder.pop()?;

        // Pop point 2 from the main stack.
        let point_2_item = stack_holder.pop()?;

        // Convert the first point to a secp point.
        let point_1 = match MaybePoint::from_slice(point_1_item.bytes()) {
            Ok(point) => point,
            Err(_) => return Err(StackError::SecpError(SecpError::InvalidSecpPoint)),
        };

        // Convert the second point to a secp point.
        let point_2 = match MaybePoint::from_slice(point_2_item.bytes()) {
            Ok(point) => point,
            Err(_) => return Err(StackError::SecpError(SecpError::InvalidSecpPoint)),
        };

        // Add the two points together.
        let addition = point_1 + point_2;

        // Convert the addition to the stack item.
        let addition_item = StackItem::new(addition.serialize_uncompressed().to_vec());

        // Push the addition back to the main stack.
        stack_holder.push(addition_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(SECPSPOINTADD_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SECPPOINTADD` opcode (0xb0).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb0]
    }
}

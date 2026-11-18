use crate::executive::stack::{
    stack_error::{SecpError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use secp::MaybePoint;
use serde::{Deserialize, Serialize};

/// Checks if a secp point is infinite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ISINFINITESECPPOINT;

/// The number of ops for the `OP_ISINFINITESECPPOINT` opcode.
impl OP_ISINFINITESECPPOINT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the point from the main stack.
        let point_item = stack_holder.last_item()?;

        // Convert the point to a secp point.
        let point = match MaybePoint::from_slice(point_item.bytes()) {
            Ok(point) => point,
            Err(_) => return Err(StackError::SecpError(SecpError::InvalidSecpPoint)),
        };

        // Check if the point is infinite.
        let result_item = match point.is_infinity() {
            true => StackItem::true_item(),
            false => StackItem::false_item(),
        };

        // Push the result back to the main stack.
        stack_holder.push(result_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_isinfinitesecppoint,
        ))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ISINFINITESECPPOINT` opcode (0xb4).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb4]
    }
}

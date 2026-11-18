use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use secp::Point;
use serde::{Deserialize, Serialize};

/// Pushes the generator point into the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_PUSHSECPGENERATORPOINT;

/// The number of ops for the `OP_PUSHSECPGENERATORPOINT` opcode.
impl OP_PUSHSECPGENERATORPOINT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the generator point.
        let generator_point = Point::generator();

        // Convert the generator point to a stack item.
        let generator_point_item =
            StackItem::new(generator_point.serialize_uncompressed().to_vec());

        // Push the generator point back to the main stack.
        stack_holder.push(generator_point_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_pushsecpgeneratorpoint,
        ))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_PUSHSECPGENERATORPOINT` opcode (0xb2).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb2]
    }
}

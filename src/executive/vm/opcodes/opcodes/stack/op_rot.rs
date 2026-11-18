use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// The 3rd item down the stack is moved to the top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ROT;

impl OP_ROT {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the third-to-top stack item.
        let third_to_top_item = stack_holder.item_by_depth(2)?;

        // Remove the third-to-top stack item.
        stack_holder.remove_item_by_depth(2)?;

        // Push the item to the stack.
        stack_holder.push(third_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(stack_holder.opcode_ops().op_rot))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ROT` opcode (0x7b).
    pub fn bytecode() -> Vec<u8> {
        vec![0x7b]
    }
}

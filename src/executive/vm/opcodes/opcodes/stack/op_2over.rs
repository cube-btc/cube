use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};
/// Copies the pair of items two spaces back in the stack to the front.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_2OVER;

impl OP_2OVER {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Clone the third-to-top stack item.
        let third_to_top_item = stack_holder.item_by_depth(2)?;

        // Clone the fourth-to-top stack item.
        let fourth_to_top_item = stack_holder.item_by_depth(3)?;

        // Push the fourth-to-top stack item to the stack.
        stack_holder.push(fourth_to_top_item)?;

        // Push the third-to-top stack item to the stack.
        stack_holder.push(third_to_top_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(stack_holder.opcode_ops().op_2over))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_2OVER` opcode (0x70).
    pub fn bytecode() -> Vec<u8> {
        vec![0x70]
    }
}

use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// The `OP_UPDATE_PARAM` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_UPDATE_PARAM;

impl OP_UPDATE_PARAM {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_update_param,
        ))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_UPDATE_PARAM` opcode (0xd3).
    pub fn bytecode() -> Vec<u8> {
        vec![0xd3]
    }
}

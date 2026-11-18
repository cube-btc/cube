use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// Puts the input onto the top of the alt stack. Removes it from the main stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_TOALTSTACK;

impl OP_TOALTSTACK {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the last item from the main stack.
        let last_item = stack_holder.pop()?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_toaltstack,
        ))?;

        // Push the last item to the alt stack.
        stack_holder.alt_stack_push(last_item)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_FROMALTSTACK` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x6b]
    }
}

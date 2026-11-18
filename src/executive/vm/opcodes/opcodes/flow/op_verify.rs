use crate::executive::stack::{
    stack_error::{MandatoryError, StackError},
    stack_holder::StackHolder,
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// Pops an item from the main stack and checks if it is true. Fails if it is not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_VERIFY;

impl OP_VERIFY {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop last from the main stack.
        let item = stack_holder.pop()?;

        // Check if the item is true.
        if item.bytes() != vec![0x01] {
            return Err(StackError::MandatoryError(
                MandatoryError::MandatoryVerifyError,
            ));
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(stack_holder.opcode_ops().op_verify))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_VERIFY` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x69]
    }
}

use crate::executive::stack::{stack_error::StackError, stack_holder::StackHolder};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// The `OP_ENDIF` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ENDIF;

impl OP_ENDIF {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // Pop the latest execution flag from the stack.
        let _flow_encounter = stack_holder
            .pop_flow_encounter()
            .ok_or(StackError::OPElseEncounteredWithoutPrecedingFlowEncounter)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(stack_holder.opcode_ops().op_endif))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ENDIF` opcode.
    pub fn bytecode() -> Vec<u8> {
        vec![0x68]
    }
}

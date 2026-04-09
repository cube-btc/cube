use crate::executive::{
    opcode::ops::OP_ELSE_OPS,
    stack::{
        flow::{flow_encounter::FlowEncounter, flow_status::FlowStatus},
        stack_error::StackError,
        stack_holder::StackHolder,
    },
};
use serde::{Deserialize, Serialize};

/// The `OP_ELSE` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_ELSE;

impl OP_ELSE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // Pop latest flow encounter
        let flow_encounter = stack_holder
            .pop_flow_encounter()
            .ok_or(StackError::OPElseEncounteredWithoutPrecedingFlowEncounter)?;

        match flow_encounter {
            FlowEncounter::IfNotif(FlowStatus::Active) => {
                // Reverse the if encounter.
                stack_holder.push_flow_encounter(FlowEncounter::Else(FlowStatus::Inactive));
            }
            FlowEncounter::IfNotif(FlowStatus::Inactive) => {
                // Push an active encounter.
                stack_holder.push_flow_encounter(FlowEncounter::Else(FlowStatus::Active));
            }
            FlowEncounter::IfNotif(FlowStatus::Uncovered) => {
                // Push an uncovered else encounter.
                stack_holder.push_flow_encounter(FlowEncounter::Else(FlowStatus::Uncovered));
            }
            _ => {
                return Err(StackError::OPElseEncounteredWithPrecedingAnotherOPElse);
            }
        }

        // Increment the ops counter.
        stack_holder.increment_ops(OP_ELSE_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_ELSE` opcode (0x67).
    pub fn bytecode() -> Vec<u8> {
        vec![0x67]
    }
}

use crate::executive::{
    opcode::ops::OP_IF_OPS,
    stack::{
        flow::{flow_encounter::FlowEncounter, flow_status::FlowStatus},
        stack_error::StackError,
        stack_holder::StackHolder,
    },
};
use serde::{Deserialize, Serialize};

/// The `OP_IF` opcode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_IF;

impl OP_IF {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // Increment the ops counter.
        stack_holder.increment_ops(OP_IF_OPS)?;

        // If this is not the active execution, return with uncovered.
        if !stack_holder.active_execution() {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Uncovered));
            return Ok(());
        }

        // Pop the latest item from the stack.
        let item = stack_holder.pop()?;

        // If the item is not true, set the active execution flag to false.
        if !item.is_true() {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Inactive));
        } else {
            stack_holder.push_flow_encounter(FlowEncounter::IfNotif(FlowStatus::Active));
        }

        Ok(())
    }

    /// Returns the bytecode for the `OP_IF` opcode (0x63).
    pub fn bytecode() -> Vec<u8> {
        vec![0x63]
    }
}

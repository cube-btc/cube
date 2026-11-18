use crate::executive::stack::{
    stack_error::StackError,
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use crate::inscriptive::params_manager::params_holder::opcode_ops_params::OpcodeOpsParams;
use serde::{Deserialize, Serialize};

/// Push the ops counter to the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_OPSCOUNTER;

/// The number of ops for the `OP_OPSCOUNTER` opcode.
impl OP_OPSCOUNTER {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the ops counter as a u32.
        let ops_counter_as_u32 = stack_holder.internal_ops_counter();

        // Convert the ops counter to a stack int.
        let ops_counter_as_stack_uint = StackUint::from_u32(ops_counter_as_u32);

        // Convert the stack int to stack item.
        let ops_counter_as_stack_item = StackItem::from_stack_uint(ops_counter_as_stack_uint);

        // Push the item to the main stack.
        stack_holder.push(ops_counter_as_stack_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OpcodeOpsParams::as_u32(
            stack_holder.opcode_ops().op_opscounter,
        ))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_OPSCOUNTER` opcode (0xbb).
    pub fn bytecode() -> Vec<u8> {
        vec![0xbb]
    }
}

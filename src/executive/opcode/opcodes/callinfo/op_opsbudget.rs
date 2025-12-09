use crate::executive::stack::{
    stack_error::StackError,
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use serde::{Deserialize, Serialize};

/// Push the ops budget to the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_OPSBUDGET;

/// The number of ops for the `OP_OPSBUDGET` opcode.
pub const OPSBUDGET_OPS: u32 = 1;

impl OP_OPSBUDGET {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the ops budget as a u32.
        let ops_budget_as_u32 = stack_holder.ops_budget();

        // Convert the ops budget to a stack int.
        let ops_budget_as_stack_uint = StackUint::from_u32(ops_budget_as_u32);

        // Convert the stack int to stack item.
        let ops_budget_as_stack_item = StackItem::from_stack_uint(ops_budget_as_stack_uint);

        // Push the item to the main stack.
        stack_holder.push(ops_budget_as_stack_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OPSBUDGET_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_OPSBUDGET` opcode (0xba).
    pub fn bytecode() -> Vec<u8> {
        vec![0xba]
    }
}

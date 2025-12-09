use crate::executive::stack::{
    stack_error::StackError,
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::{SafeConverter, StackItemUintExt, StackUint},
};
use serde::{Deserialize, Serialize};

/// Push the ops price to the stack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_OPSPRICE;

/// The number of ops for the `OP_OPSPRICE` opcode.
pub const OPSPRICE_OPS: u32 = 1;

impl OP_OPSPRICE {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Get the ops price as a u32.
        let ops_price_as_u32 = stack_holder.ops_price();

        // Convert the ops price to a stack int.
        let ops_price_as_stack_uint = StackUint::from_u32(ops_price_as_u32);

        // Convert the stack int to stack item.
        let ops_price_as_stack_item = StackItem::from_stack_uint(ops_price_as_stack_uint);

        // Push the item to the main stack.
        stack_holder.push(ops_price_as_stack_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(OPSPRICE_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_OPSPRICE` opcode (0xbc).
    pub fn bytecode() -> Vec<u8> {
        vec![0xbc]
    }
}

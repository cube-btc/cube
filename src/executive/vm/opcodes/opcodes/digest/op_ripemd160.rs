use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use bitcoin::hashes::ripemd160;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

/// The input is hashed using RIPEMD-160.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_RIPEMD160;

/// The number of ops for the `OP_RIPEMD160` opcode.
pub const RIPEMD160_OPS: u32 = 30;

impl OP_RIPEMD160 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Hash the item using RIPEMD-160.
        let hash = ripemd160::Hash::hash(preimage.bytes())
            .to_byte_array()
            .to_vec();

        // Increment the ops counter.
        stack_holder.increment_ops(RIPEMD160_OPS)?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(hash))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_RIPEMD160` opcode (0xa6).
    pub fn bytecode() -> Vec<u8> {
        vec![0xa6]
    }
}

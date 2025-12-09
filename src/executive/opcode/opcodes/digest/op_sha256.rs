use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use bitcoin::hashes::sha256;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

/// The input is hashed using SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_SHA256;

/// The number of ops for the `OP_SHA256` opcode.
pub const SHA256_OPS: u32 = 42;

impl OP_SHA256 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Hash the item using SHA-256.
        let hash = sha256::Hash::hash(preimage.bytes())
            .to_byte_array()
            .to_vec();

        // Increment the ops counter.
        stack_holder.increment_ops(SHA256_OPS)?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(hash))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_SHA256` opcode (0xa8).
    pub fn bytecode() -> Vec<u8> {
        vec![0xa8]
    }
}

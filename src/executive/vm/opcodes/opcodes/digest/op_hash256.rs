use crate::executive::stack::{
    stack_error::StackError, stack_holder::StackHolder, stack_item::StackItem,
};
use bitcoin::hashes::sha256;
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

/// The input is hashed two times with SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_HASH256;

/// The number of ops for the `OP_HASH256` opcode.
pub const HASH256_OPS: u32 = 84;

impl OP_HASH256 {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the preimage from the main stack.
        let preimage = stack_holder.pop()?;

        // Hash the item using SHA-256.
        let first_hash = sha256::Hash::hash(preimage.bytes())
            .to_byte_array()
            .to_vec();

        // Hash the item, again, using SHA-256.
        let second_hash = sha256::Hash::hash(&first_hash).to_byte_array().to_vec();

        // Increment the ops counter.
        stack_holder.increment_ops(HASH256_OPS)?;

        // Push the hash back to the main stack.
        stack_holder.push(StackItem::new(second_hash))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_HASH256` opcode (0xaa).
    pub fn bytecode() -> Vec<u8> {
        vec![0xaa]
    }
}

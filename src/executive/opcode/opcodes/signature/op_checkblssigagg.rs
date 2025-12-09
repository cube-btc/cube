use crate::executive::stack::{
    stack_error::{BLSError, StackError, StackUintError},
    stack_holder::StackHolder,
    stack_item::StackItem,
    stack_uint::StackItemUintExt,
};
use crate::transmutative::bls::{key::BLSPublicKey, verify::bls_verify_aggregate};
use serde::{Deserialize, Serialize};

/// Checks a BLS aggregate signature against a set of keys and messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CHECKBLSSIGAGG;

impl OP_CHECKBLSSIGAGG {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop the count from the stack.
        let count = stack_holder.pop()?;

        // Convert the count to a stack u32.
        let count = count
            .to_stack_uint()
            .ok_or(StackError::StackUintError(
                StackUintError::StackUintConversionError,
            ))?
            .as_usize();

        // Collect the keys.
        let mut keys = Vec::<BLSPublicKey>::new();
        for _ in 0..count {
            // Pop public key from the stack.
            let public_key = stack_holder.pop()?;

            // Convert the public key to 48 bytes.
            let public_key: BLSPublicKey = public_key
                .bytes()
                .try_into()
                .map_err(|_| StackError::BLSError(BLSError::InvalidBLSPublicKeyBytes))?;

            // Push the public key to the vector.
            keys.push(public_key);
        }

        // Collect the messages.
        let mut messages = Vec::<[u8; 32]>::new();
        for _ in 0..count {
            // Pop message from the stack.
            let message = stack_holder.pop()?;

            // Convert the message to 32 bytes.
            let message: [u8; 32] = message
                .bytes()
                .try_into()
                .map_err(|_| StackError::BLSError(BLSError::InvalidBLSMessageBytes))?;

            // Push the message to the vector.
            messages.push(message);
        }

        // Pop the aggregate signature item from the stack.
        let aggregate_signature_item = stack_holder.pop()?;

        // Convert the aggregate signature item to 96 bytes.
        let aggregate_signature: [u8; 96] = aggregate_signature_item
            .bytes()
            .try_into()
            .map_err(|_| StackError::BLSError(BLSError::InvalidBLSSignatureBytes))?;

        // Verify the signature.
        let verify_result = bls_verify_aggregate(keys, messages, aggregate_signature);

        // Match the result.
        let result_item = match verify_result {
            true => StackItem::true_item(),
            false => StackItem::false_item(),
        };

        // Push the results to the main stack.
        stack_holder.push(result_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(calculate_ops(count as u32))?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_CHECKBLSSIGAGG` opcode (0xb8).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb8]
    }
}

const CHECKBLSSIGAGG_OPS_BASE: u32 = 100;
const CHECKBLSSIGAGG_OPS_MULTIPLIER: u32 = 50;

// Calculate the number of ops for a CHECKBLSSIGAGG opcode.
fn calculate_ops(count: u32) -> u32 {
    // Return the number of ops.
    CHECKBLSSIGAGG_OPS_BASE + (CHECKBLSSIGAGG_OPS_MULTIPLIER * count)
}

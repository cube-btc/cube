use crate::executive::stack::{
    stack_error::{BLSError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use crate::transmutative::bls::{key::BLSPublicKey, verify::bls_verify};
use serde::{Deserialize, Serialize};

/// Checks a BLS signature according against a key and a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CHECKBLSSIG;

/// The number of ops for the `OP_CHECKBLSSIG` opcode.
pub const CHECKBLSSIG_OPS: u32 = 100;

impl OP_CHECKBLSSIG {
    pub fn execute(stack_holder: &mut StackHolder) -> Result<(), StackError> {
        // If this is not the active execution, return immediately.
        if !stack_holder.active_execution() {
            return Ok(());
        }

        // Pop public key from the stack.
        let public_key = stack_holder.pop()?;

        // Pop message from the stack.
        let message = stack_holder.pop()?;

        // Pop signature from the stack.
        let signature = stack_holder.pop()?;

        // Convert the public key to 48 bytes.
        let public_key: BLSPublicKey = public_key
            .bytes()
            .try_into()
            .map_err(|_| StackError::BLSError(BLSError::InvalidBLSPublicKeyBytes))?;

        // Convert the message to 32 bytes.
        let message: [u8; 32] = message
            .bytes()
            .try_into()
            .map_err(|_| StackError::BLSError(BLSError::InvalidBLSSignatureBytes))?;

        // Convert the signature to 96 bytes.
        let signature: [u8; 96] = signature
            .bytes()
            .try_into()
            .map_err(|_| StackError::BLSError(BLSError::InvalidBLSSignatureBytes))?;

        // Verify the signature.
        let verify_result = bls_verify(&public_key, message, signature);

        // Match the result.
        let result_item = match verify_result {
            true => StackItem::true_item(),
            false => StackItem::false_item(),
        };

        // Push the results to the main stack.
        stack_holder.push(result_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(CHECKBLSSIG_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_CHECKBLSSIG` opcode (0xb7).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb7]
    }
}

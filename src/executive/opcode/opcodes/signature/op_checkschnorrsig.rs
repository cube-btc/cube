use crate::executive::stack::{
    stack_error::{SchnorrError, StackError},
    stack_holder::StackHolder,
    stack_item::StackItem,
};
use crate::transmutative::secp::schnorr::{self, SchnorrSigningMode};
use serde::{Deserialize, Serialize};

/// Checks a schnorr signature according to the 'Cube/challenge' tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub struct OP_CHECKSCHNORRSIG;

/// The number of ops for the `OP_CHECKSCHNORRSIG` opcode.
pub const CHECKSCHNORRSIG_OPS: u32 = 100;

impl OP_CHECKSCHNORRSIG {
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

        // Convert public key to bytes.
        // NOTE: This can be a 32-byte x-only or a 33-byte compressed public key.
        let public_key_bytes: Vec<u8> = public_key.bytes().to_vec();

        // Convert message to bytes.
        let message_bytes: [u8; 32] = message
            .bytes()
            .try_into()
            .map_err(|_| StackError::SchnorrError(SchnorrError::InvalidSchnorrMessageBytes))?;

        // Convert signature to bytes.
        let signature_bytes: [u8; 64] = signature
            .bytes()
            .try_into()
            .map_err(|_| StackError::SchnorrError(SchnorrError::InvalidSchnorrSignatureBytes))?;

        // Match public key length.
        let verify_result = match public_key_bytes.len() {
            32 => {
                // Convert public key to bytes.
                let public_key_bytes: [u8; 32] = public_key_bytes.try_into().map_err(|_| {
                    StackError::SchnorrError(SchnorrError::InvalidSchnorrPublicKeyBytes)
                })?;

                // Verify the signature.
                schnorr::verify_xonly(
                    public_key_bytes,
                    message_bytes,
                    signature_bytes,
                    SchnorrSigningMode::Cube,
                )
            }
            33 => {
                // Convert public key to bytes.
                let public_key_bytes: [u8; 33] = public_key_bytes.try_into().map_err(|_| {
                    StackError::SchnorrError(SchnorrError::InvalidSchnorrPublicKeyBytes)
                })?;

                // Verify the signature.
                schnorr::verify_compressed(
                    public_key_bytes,
                    message_bytes,
                    signature_bytes,
                    SchnorrSigningMode::Cube,
                )
            }
            65 => {
                // Convert public key to bytes.
                let public_key_bytes: [u8; 65] = public_key_bytes.try_into().map_err(|_| {
                    StackError::SchnorrError(SchnorrError::InvalidSchnorrPublicKeyBytes)
                })?;

                // Verify the signature.
                schnorr::verify_uncompressed(
                    public_key_bytes,
                    message_bytes,
                    signature_bytes,
                    SchnorrSigningMode::Cube,
                )
            }
            _ => {
                return Err(StackError::SchnorrError(
                    SchnorrError::InvalidSchnorrPublicKeyBytes,
                ))
            }
        };
        // Get the result item.
        let result_item = match verify_result {
            true => StackItem::true_item(),
            false => StackItem::false_item(),
        };

        // Push the results to the main stack.
        stack_holder.push(result_item)?;

        // Increment the ops counter.
        stack_holder.increment_ops(CHECKSCHNORRSIG_OPS)?;

        Ok(())
    }

    /// Returns the bytecode for the `OP_CHECKSCHNORRSIG` opcode (0xb5).
    pub fn bytecode() -> Vec<u8> {
        vec![0xb5]
    }
}

use super::key::bls_public_key_bytes_to_bls_public_key;
use crate::inscriptive::baked;
use bls_on_arkworks as bls;

/// Verify a BLS signature.
///
/// # Arguments
///
/// * `public_key` - The BLS public key (48-byte compressed encoding).
/// * `message` - The message to verify.
/// * `signature` - The signature to verify.
pub fn bls_verify(public_key: &[u8; 48], message: [u8; 32], signature: [u8; 96]) -> bool {
    let Some(public_key) = bls_public_key_bytes_to_bls_public_key(*public_key) else {
        return false;
    };

    // Get the message tag.
    let message_tag = format!("{}/{}", baked::PROJECT_TAG, "bls/message")
        .as_bytes()
        .to_vec();

    // Verify the signature.
    bls::verify(
        &public_key,
        &message.to_vec(),
        &signature.to_vec(),
        &message_tag,
    )
}

/// Verify a BLS aggregate signature.
///
/// # Arguments
///
/// * `public_keys` - The BLS public keys (48-byte compressed encodings).
/// * `messages` - The messages to verify.
/// * `aggregate_signature` - The aggregate signature to verify.
pub fn bls_verify_aggregate(
    public_keys: Vec<[u8; 48]>,
    messages: Vec<[u8; 32]>,
    aggregate_signature: [u8; 96],
) -> bool {
    let mut keys = Vec::with_capacity(public_keys.len());
    for pk in public_keys {
        let Some(pk) = bls_public_key_bytes_to_bls_public_key(pk) else {
            return false;
        };
        keys.push(pk);
    }

    // Get the message tag.
    let message_tag = format!("{}/{}", baked::PROJECT_TAG, "bls/message")
        .as_bytes()
        .to_vec();

    // Messages as vectors of octets.
    let messages = messages
        .into_iter()
        .map(|m| m.to_vec())
        .collect::<Vec<Vec<u8>>>();

    // Verify the aggregate signature.
    bls::aggregate_verify(
        keys,
        messages,
        &aggregate_signature.to_vec(),
        &message_tag,
    )
}

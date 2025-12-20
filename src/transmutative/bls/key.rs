use crate::transmutative::hash::{Hash, HashTag};
use bls_on_arkworks::{
    self as bls,
    types::{PublicKey, SecretKey},
};

/// The type of a BLS public key.
pub type BLSPublicKey = PublicKey;

/// The type of a BLS secret key.
pub type BLSSecretKey = SecretKey;

/// Converts 32-byte Secp256k1 secret key bytes to 48-byte BLS secret key bytes.
///
/// # Arguments
///
/// * `secp_secret_bytes` - A 32-byte Secp256k1 secret key bytes.
///
pub fn secp_secret_key_bytes_to_bls_secret_key_bytes(secp_secret_key_bytes: &[u8; 32]) -> [u8; 48] {
    // 1 Hash the Secp256k1 secret key bytes with the BLSSecretKey tag.
    let tagged_long_hash_of_secp_secret_key_bytes: [u8; 64] = secp_secret_key_bytes
        .to_vec()
        .long_hash(Some(HashTag::BLSSecretKey));

    // 2 Take first 48 bytes of the long 64-byte hash.
    let bls_secret_key_bytes: [u8; 48] = tagged_long_hash_of_secp_secret_key_bytes[..48]
        .try_into()
        .expect("This should never happen.");

    // 3 Return the 48-byte BLS secret key bytes.
    bls_secret_key_bytes
}

/// Converts 48-byte BLS secret key bytes to a BLS secret key.
///
/// # Arguments
///
/// * `secret_key_bytes` - A 48-byte BLS secret key.
///
pub fn bls_secret_key_bytes_to_bls_secret_key(secret_key_bytes: [u8; 48]) -> BLSSecretKey {
    // 1 Convert the 48-byte secret key to a BLS secret key.
    bls::os2ip(&secret_key_bytes.to_vec())
}

/// Converts 48-byte BLS public key bytes to a BLS public key.
///
/// # Arguments
///
/// * `public_key_bytes` - A 48-byte BLS public key.
///
pub fn bls_public_key_bytes_to_bls_public_key(public_key_bytes: [u8; 48]) -> Option<BLSPublicKey> {
    // 1 Convert the 48-byte public key to a BLS public key.
    public_key_bytes.try_into().ok()
}

/// Converts a BLS secret key to a BLS public key.
///
/// # Arguments
///
/// * `secret_key` - A BLS secret key.
///
pub fn bls_secret_key_to_bls_public_key(secret_key: BLSSecretKey) -> Option<BLSPublicKey> {
    // 1 Convert the BLS secret key to a BLS public key.
    bls::sk_to_pk(secret_key).try_into().ok()
}

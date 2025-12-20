use super::bls::key::{
    bls_public_key_bytes_to_bls_public_key, bls_secret_key_bytes_to_bls_secret_key,
    bls_secret_key_to_bls_public_key, secp_secret_key_bytes_to_bls_secret_key_bytes, BLSPublicKey,
    BLSSecretKey,
};
use crate::transmutative::secp::schnorr::Bytes32;
use bech32::{Bech32, Hrp};
use libc;
use secp::{Point, Scalar};
use zeroize::Zeroize;

/// A secure wrapper for 32-byte secret key bytes that prevents accidental exposure
/// and automatically zeroizes memory on drop.
///
/// This type is intentionally NOT Clone to prevent multiple copies of secrets in memory.
struct SecretBytes32 {
    bytes: [u8; 32],
}

impl SecretBytes32 {
    // 1 Create a new SecretBytes32 instance.
    fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    // 2 Expose the secret bytes. Use with extreme caution.
    fn expose_secret(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl Zeroize for SecretBytes32 {
    // 1 Zeroize the secret bytes.
    fn zeroize(&mut self) {
        self.bytes.zeroize();
    }
}

impl Drop for SecretBytes32 {
    // 1 Automatically zeroize memory on drop.
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// A secure wrapper for 48-byte secret key bytes that prevents accidental exposure
/// and automatically zeroizes memory on drop.
///
/// This type is intentionally NOT Clone to prevent multiple copies of secrets in memory.
struct SecretBytes48 {
    bytes: [u8; 48],
}

impl SecretBytes48 {
    // 1 Create a new SecretBytes48 instance.
    fn new(bytes: [u8; 48]) -> Self {
        Self { bytes }
    }

    // 2 Expose the secret bytes. Use with extreme caution.
    fn expose_secret(&self) -> &[u8; 48] {
        &self.bytes
    }
}

impl Zeroize for SecretBytes48 {
    // 1 Zeroize the secret bytes.
    fn zeroize(&mut self) {
        self.bytes.zeroize();
    }
}

impl Drop for SecretBytes48 {
    // 1 Automatically zeroize memory on drop.
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// A secure key holder that stores cryptographic keys in memory with automatic zeroization.
///
/// This struct implements best security practices:
/// - Secret keys are wrapped in secure wrappers to prevent accidental exposure
/// - Memory is locked using `mlock` to prevent swapping to disk
/// - All sensitive fields are automatically zeroed on drop
/// - Not `Clone` to prevent multiple copies of secrets in memory
/// - `Send + Sync` safe for use in async contexts via `Arc<KeyHolder>`
///
/// # Thread Safety
///
/// `KeyHolder` is safe to share across threads via `Arc<KeyHolder>` since all
/// public methods are read-only. The internal memory locking operations are
/// thread-safe as they operate on fixed memory addresses.
pub struct KeyHolder {
    // Schnorr/Secp256k1
    secp_secret_key_bytes: SecretBytes32,
    secp_public_key_bytes: [u8; 32],
    // BLS-381
    bls_secret_key_bytes: SecretBytes48,
    bls_public_key_bytes: [u8; 48],
    // Track if memory is locked for cleanup
    memory_locked: bool,
}

impl KeyHolder {
    /// Creates a new `KeyHolder` from 32-byte secret key bytes.
    ///
    /// The secret key bytes are immediately wrapped in secure wrappers to prevent accidental exposure.
    /// All sensitive data will be automatically zeroed when the `KeyHolder` is dropped.
    /// Memory is locked using `mlock` to prevent swapping to disk.
    ///
    /// Returns `None` if the secret key is invalid.
    pub fn new(secret_key_bytes: [u8; 32]) -> Option<Self> {
        // 1 Wrap secp256k1 secret key bytes immediately to prevent accidental exposure.
        let secp_secret_key_bytes = SecretBytes32::new(secret_key_bytes);

        // 2 Access the secret key bytes for processing.
        let secp_secret_key_bytes_ref = secp_secret_key_bytes.expose_secret();

        // 3 Validate and compute secp256k1 public key.
        // 3.1 Convert secret key bytes to scalar.
        let secp_scalar = match Scalar::from_slice(secp_secret_key_bytes_ref) {
            Ok(scalar) => scalar,
            Err(_) => return None,
        };
        // 3.2 Compute the public key point.
        let mut secp_public_key_point = secp_scalar.base_point_mul();
        // 3.3 Normalize the point to even parity.
        secp_public_key_point = secp_public_key_point.negate_if(secp_public_key_point.parity());
        // 3.4 Serialize to 32 bytes (x-only public key).
        let secp_public_key_bytes = secp_public_key_point.serialize_xonly();

        // 4 Convert secp256k1 secret key to BLS secret key.
        // 4.1 Convert secp256k1 secret key bytes to BLS secret key bytes (48 bytes).
        let mut bls_secret_key_bytes_ =
            secp_secret_key_bytes_to_bls_secret_key_bytes(secp_secret_key_bytes_ref);
        // 4.2 Wrap BLS secret key bytes in secure wrapper.
        let bls_secret_key_bytes = SecretBytes48::new(bls_secret_key_bytes_);

        // 5 Compute BLS public key from BLS secret key.
        // 5.1 Convert BLS secret key bytes to BLS secret key.
        let bls_secret_key = bls_secret_key_bytes_to_bls_secret_key(bls_secret_key_bytes_);
        // 5.1.1 Zeroize the temporary BLS secret key bytes immediately after use.
        bls_secret_key_bytes_.zeroize();
        // 5.2 Compute BLS public key from BLS secret key.
        let bls_public_key: BLSPublicKey = match bls_secret_key_to_bls_public_key(bls_secret_key) {
            Some(bls_public_key) => bls_public_key,
            None => return None,
        };
        // 5.3 Serialize BLS public key to 48 bytes.
        let bls_public_key_bytes: [u8; 48] = match bls_public_key.try_into() {
            Ok(bls_public_key_bytes) => bls_public_key_bytes,
            Err(_) => return None,
        };

        // 6 Construct the key holder.
        let mut key_holder = KeyHolder {
            secp_secret_key_bytes,
            secp_public_key_bytes,
            bls_secret_key_bytes,
            bls_public_key_bytes,
            memory_locked: false,
        };

        // 7 Lock memory to prevent swapping to disk.
        key_holder.lock_memory();

        // 8 Return the key holder.
        Some(key_holder)
    }

    /// Locks the memory containing sensitive data to prevent swapping to disk.
    ///
    /// This uses `mlock` to prevent the OS from paging sensitive memory to disk.
    ///
    /// # Security Note
    ///
    /// If memory locking fails, a warning is logged but execution continues.
    /// This is a best-effort security measure; the system may not have sufficient
    /// privileges or resources to lock memory.
    fn lock_memory(&mut self) {
        // 1 Lock secp256k1 secret key bytes.
        let schnorr_bytes = self.secp_secret_key_bytes.expose_secret();
        unsafe {
            if libc::mlock(
                schnorr_bytes.as_ptr() as *const libc::c_void,
                schnorr_bytes.len(),
            ) != 0
            {
                // Memory locking failed - log warning but continue
                // This can happen due to insufficient privileges or resource limits
                eprintln!("Warning: Failed to lock secp256k1 secret key memory (mlock failed)");
            }
        }

        // 2 Lock BLS secret key bytes.
        let bls_bytes = self.bls_secret_key_bytes.expose_secret();
        unsafe {
            if libc::mlock(bls_bytes.as_ptr() as *const libc::c_void, bls_bytes.len()) != 0 {
                // Memory locking failed - log warning but continue
                eprintln!("Warning: Failed to lock BLS secret key memory (mlock failed)");
            }
        }

        // 3 Mark memory as locked.
        self.memory_locked = true;
    }

    /// Unlocks the memory and zeroizes sensitive data.
    ///
    /// This is called automatically when the KeyHolder is dropped.
    /// Note: SecretBytes32 and SecretBytes48 will automatically zeroize on drop, but we unlock memory here.
    fn unlock_and_zeroize(&mut self) {
        // 1 Check if memory is locked.
        if !self.memory_locked {
            return;
        }

        // 2 Unlock secp256k1 secret key bytes.
        let schnorr_bytes = self.secp_secret_key_bytes.expose_secret();
        unsafe {
            if libc::munlock(
                schnorr_bytes.as_ptr() as *const libc::c_void,
                schnorr_bytes.len(),
            ) != 0
            {
                // Unlocking failed - this is non-critical but log for debugging
                eprintln!("Warning: Failed to unlock secp256k1 secret key memory (munlock failed)");
            }
        }

        // 3 Unlock BLS secret key bytes.
        let bls_bytes = self.bls_secret_key_bytes.expose_secret();
        unsafe {
            if libc::munlock(bls_bytes.as_ptr() as *const libc::c_void, bls_bytes.len()) != 0 {
                // Unlocking failed - this is non-critical but log for debugging
                eprintln!("Warning: Failed to unlock BLS secret key memory (munlock failed)");
            }
        }

        // 4 Zeroize explicitly (though they will also zeroize on drop).
        self.secp_secret_key_bytes.zeroize();
        self.bls_secret_key_bytes.zeroize();

        // 5 Mark memory as unlocked.
        self.memory_locked = false;
    }

    /// Returns the 32-byte schnorr secret key.
    ///
    /// # Security Warning
    ///
    /// This method exposes the secret key bytes. Use with extreme caution.
    /// The returned value should be zeroized after use if possible.
    pub fn secp_secret_key_bytes(&self) -> [u8; 32] {
        // 1 Expose and return the secp256k1 secret key bytes.
        *self.secp_secret_key_bytes.expose_secret()
    }

    /// Returns the schnorr secret key as a Scalar.
    ///
    /// # Security Warning
    ///
    /// This method exposes the secret key. Use with extreme caution.
    pub fn secp_secret_key_scalar(&self) -> Scalar {
        // 1 Convert secp256k1 secret key bytes to scalar.
        Scalar::from_slice(self.secp_secret_key_bytes.expose_secret())
            .expect("This should never happen.")
    }

    /// Returns the schnorr public key as 32 bytes (x-only format).
    pub fn secp_public_key_bytes(&self) -> [u8; 32] {
        // 1 Return the secp256k1 public key bytes (x-only format).
        self.secp_public_key_bytes
    }

    /// Returns the schnorr public key as a Point.
    pub fn secp_public_key_point(&self) -> Point {
        // 1 Reconstruct Point from x-only bytes.
        // 1.1 Create a 33-byte buffer with even point prefix.
        let mut point_bytes = Vec::with_capacity(33);
        point_bytes.push(0x02);
        point_bytes.extend_from_slice(&self.secp_public_key_bytes);

        // 1.2 Parse the point (this should always work for valid keys).
        use secp::MaybePoint;
        match MaybePoint::from_slice(&point_bytes) {
            Ok(MaybePoint::Valid(point)) => point,
            Ok(MaybePoint::Infinity) => panic!("Public key is infinity (should never happen)"),
            Err(_) => panic!("Failed to reconstruct Point from bytes"),
        }
    }

    /// Returns the 48-byte BLS secret key.
    ///
    /// # Security Warning
    ///
    /// This method exposes the secret key bytes. Use with extreme caution.
    /// The returned value should be zeroized after use if possible.
    pub fn bls_secret_key_bytes(&self) -> [u8; 48] {
        // 1 Expose and return the BLS secret key bytes.
        *self.bls_secret_key_bytes.expose_secret()
    }

    /// Returns the BLS secret key.
    ///
    /// # Security Warning
    ///
    /// This method exposes the secret key. Use with extreme caution.
    pub fn bls_secret_key(&self) -> BLSSecretKey {
        // 1 Get the BLS secret key bytes.
        let secret_key_bytes = self.bls_secret_key_bytes();
        // 2 Convert BLS secret key bytes to BLS secret key.
        bls_secret_key_bytes_to_bls_secret_key(secret_key_bytes)
    }

    /// Returns the BLS public key.
    pub fn bls_public_key_bytes(&self) -> [u8; 48] {
        // 1 Return the BLS public key bytes.
        self.bls_public_key_bytes
    }

    /// Returns the BLS public key.
    pub fn bls_public_key(&self) -> BLSPublicKey {
        // 1 Get the BLS public key bytes.
        let public_key_bytes = self.bls_public_key_bytes();
        // 2 Convert BLS public key bytes to BLS public key.
        bls_public_key_bytes_to_bls_public_key(public_key_bytes).expect("This should never happen.")
    }

    /// Returns the Nostr keypair, computed from the schnorr secret key.
    ///
    /// This function computes the keypair on-demand rather than storing it.
    ///
    /// # Panics
    ///
    /// Panics if the keypair cannot be computed (should never happen for valid keys).
    pub fn nostr_key_pair(&self) -> nostr_sdk::Keys {
        // 1 Get the secp256k1 secret key as bytes.
        let secret_bytes = self.secp_secret_key_bytes.expose_secret();

        // 2 Convert secret key bytes to scalar.
        let scalar =
            Scalar::from_slice(secret_bytes).expect("Invalid secret key bytes in KeyHolder");

        // 3 Serialize scalar and convert to nsec.
        let nsec = scalar
            .serialize()
            .to_nsec()
            .expect("Failed to convert secret key to nsec");

        // 4 Parse the nsec to Nostr keypair.
        nostr_sdk::Keys::parse(&nsec).expect("Failed to parse nsec to nostr keypair")
    }

    /// Returns the Nostr `npub` string (compatibility method).
    ///
    /// # Panics
    ///
    /// Panics if the public key cannot be converted to npub (should never happen for valid keys).
    pub fn npub(&self) -> String {
        // 1 Convert secp256k1 public key bytes to npub string.
        self.secp_public_key_bytes
            .to_npub()
            .expect("Failed to convert public key to npub")
    }
}

// KeyHolder is intentionally NOT Clone to prevent multiple copies of secrets in memory.
// Use Arc<KeyHolder> for shared ownership across async tasks instead.
//
// This is a security best practice: having multiple copies of secret keys increases
// the attack surface. Sharing a single instance via Arc is safer and more efficient.

// KeyHolder is Send + Sync safe:
// - Send: Safe to transfer between threads (memory locking is per-instance, no shared mutable state)
// - Sync: Safe to share via &KeyHolder across threads (all public methods are read-only)
unsafe impl Send for KeyHolder {}
unsafe impl Sync for KeyHolder {}

impl Drop for KeyHolder {
    /// Automatically zeroizes all sensitive data and unlocks memory when dropped.
    ///
    /// This ensures that secret keys are securely erased from memory and cannot
    /// be recovered after the KeyHolder is dropped.
    fn drop(&mut self) {
        // 1 Unlock memory and zeroize all sensitive data.
        self.unlock_and_zeroize();
    }
}

/// Trait for converting 32-byte keys into Bech32-encoded `nsec` or `npub` strings.
pub trait ToNostrKeyStr {
    /// Converts a 32-byte secret key into a Bech32-encoded `nsec` string.
    ///
    /// Returns `None` if the key is invalid.
    fn to_nsec(&self) -> Option<String>;

    /// Converts a 32-byte public key into a Bech32-encoded `npub` string.
    ///
    /// Returns `None` if the key is invalid.
    fn to_npub(&self) -> Option<String>;
}

/// Trait for decoding Bech32-encoded `nsec` or `npub` strings into 32-byte keys.
pub trait FromNostrKeyStr {
    /// Decodes a Bech32-encoded `nsec` string into a 32-byte secret key.
    ///
    /// Returns `None` if the string is invalid or doesn't represent a valid secret key.
    fn from_nsec(&self) -> Option<[u8; 32]>;

    /// Decodes a Bech32-encoded `npub` string into a 32-byte public key.
    ///
    /// Returns `None` if the string is invalid or doesn't represent a valid public key.
    fn from_npub(&self) -> Option<[u8; 32]>;
}

impl ToNostrKeyStr for [u8; 32] {
    fn to_nsec(&self) -> Option<String> {
        // 1 Validate that the bytes represent a valid secret key.
        if !self.is_valid_secret() {
            return None;
        }

        // 2 Parse the "nsec" human-readable part.
        let hrp = match Hrp::parse("nsec") {
            Ok(hrp) => hrp,
            Err(_) => return None,
        };

        // 3 Encode the secret key bytes as a Bech32 nsec string.
        let nsec = match bech32::encode::<Bech32>(hrp, self) {
            Ok(encoded) => encoded,
            Err(_) => return None,
        };

        // 4 Return the nsec string.
        Some(nsec)
    }

    fn to_npub(&self) -> Option<String> {
        // 1 Validate that the bytes represent a valid public key.
        if !self.is_valid_public() {
            return None;
        }

        // 2 Parse the "npub" human-readable part.
        let hrp = match Hrp::parse("npub") {
            Ok(hrp) => hrp,
            Err(_) => return None,
        };

        // 3 Encode the public key bytes as a Bech32 npub string.
        let npub = match bech32::encode::<Bech32>(hrp, self) {
            Ok(encoded) => encoded,
            Err(_) => return None,
        };

        // 4 Return the npub string.
        Some(npub)
    }
}

impl FromNostrKeyStr for &str {
    fn from_nsec(&self) -> Option<[u8; 32]> {
        // 1 Decode the Bech32 string.
        let (hrp, decoded_bytes) = match bech32::decode(self) {
            Ok(decoded) => decoded,
            Err(_) => return None,
        };

        // 2 Validate that the human-readable part is "nsec".
        if hrp.as_str() != "nsec" {
            return None;
        }

        // 3 Validate that the decoded bytes length is 32.
        if decoded_bytes.len() != 32 {
            return None;
        }

        // 4 Convert decoded bytes to a 32-byte array.
        let secret_key: [u8; 32] = decoded_bytes.try_into().ok()?;

        // 5 Validate that the bytes represent a valid secret key.
        if !secret_key.is_valid_secret() {
            return None;
        }

        // 6 Return the secret key bytes.
        Some(secret_key)
    }

    fn from_npub(&self) -> Option<[u8; 32]> {
        // 1 Decode the Bech32 string.
        let (hrp, decoded_bytes) = match bech32::decode(self) {
            Ok(decoded) => decoded,
            Err(_) => return None,
        };

        // 2 Validate that the human-readable part is "npub".
        if hrp.as_str() != "npub" {
            return None;
        }

        // 3 Validate that the decoded bytes length is 32.
        if decoded_bytes.len() != 32 {
            return None;
        }

        // 4 Convert decoded bytes to a 32-byte array.
        let public_key: [u8; 32] = decoded_bytes.try_into().ok()?;

        // 5 Validate that the bytes represent a valid public key.
        if !public_key.is_valid_public() {
            return None;
        }

        // 6 Return the public key bytes.
        Some(public_key)
    }
}

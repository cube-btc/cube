use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// BLS key of an account.
type BLSKey = [u8; 48];

/// Schnorr signature.
type SchnorrSignature = [u8; 64];

/// BLS signature.
type BLSSignature = [u8; 96];

/// Helper function to serialize [u8; 48] as a byte vector.
pub fn serialize_bls_key<S>(bls_key: &BLSKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    bls_key.as_slice().serialize(serializer)
}

/// Helper function to deserialize [u8; 48] from a byte vector.
pub fn deserialize_bls_key<'de, D>(deserializer: D) -> Result<BLSKey, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
    if bytes.len() == 48 {
        let mut array = [0u8; 48];
        array.copy_from_slice(&bytes);
        Ok(array)
    } else {
        Err(serde::de::Error::custom(format!(
            "BLS key must be exactly 48 bytes, got {} bytes",
            bytes.len()
        )))
    }
}

/// Helper function to serialize [u8; 64] as a byte vector.
pub fn serialize_schnorr_signature<S>(
    schnorr_signature: &SchnorrSignature,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    schnorr_signature.as_slice().serialize(serializer)
}

/// Helper function to deserialize [u8; 64] from a byte vector.
pub fn deserialize_schnorr_signature<'de, D>(
    deserializer: D,
) -> Result<SchnorrSignature, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
    if bytes.len() == 64 {
        let mut array = [0u8; 64];
        array.copy_from_slice(&bytes);
        Ok(array)
    } else {
        Err(serde::de::Error::custom(format!(
            "Schnorr signature must be exactly 64 bytes, got {} bytes",
            bytes.len()
        )))
    }
}

/// Helper function to serialize [u8; 96] as a byte vector.
pub fn serialize_bls_signature<S>(
    bls_signature: &BLSSignature,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    bls_signature.as_slice().serialize(serializer)
}

/// Helper function to deserialize [u8; 96] from a byte vector.
pub fn deserialize_bls_signature<'de, D>(deserializer: D) -> Result<BLSSignature, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
    if bytes.len() == 96 {
        let mut array = [0u8; 96];
        array.copy_from_slice(&bytes);
        Ok(array)
    } else {
        Err(serde::de::Error::custom(format!(
            "BLS signature must be exactly 96 bytes, got {} bytes",
            bytes.len()
        )))
    }
}

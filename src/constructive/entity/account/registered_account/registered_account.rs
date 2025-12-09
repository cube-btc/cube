use super::flame_config::flame_config::FlameConfig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

/// BLS key of an account.
type BLSKey = [u8; 48];

/// A registered and possibly configured account.
#[derive(Clone, Serialize, Deserialize)]
pub struct RegisteredAccount {
    /// The secp256k1 public key of the account.
    pub key: [u8; 32],

    /// The registery index of the account.
    pub registery_index: u64,

    /// The rank of the account.
    pub rank: Option<u64>,

    /// The BLS key of the account.
    #[serde(
        serialize_with = "serialize_bls_key",
        deserialize_with = "deserialize_bls_key"
    )]
    pub bls_key: Option<BLSKey>,

    /// The secondary aggregation key of the account.
    pub secondary_aggregation_key: Option<Vec<u8>>,

    /// The flame config of the account.
    pub flame_config: Option<FlameConfig>,
}

impl RegisteredAccount {
    /// Constructs a new registered account.
    pub fn new(
        key: [u8; 32],
        registery_index: u64,
        rank: Option<u64>,
        bls_key: Option<[u8; 48]>,
        secondary_aggregation_key: Option<Vec<u8>>,
        flame_config: Option<FlameConfig>,
    ) -> Self {
        Self {
            key,
            registery_index,
            rank,
            bls_key,
            secondary_aggregation_key,
            flame_config,
        }
    }
    pub fn json(&self) -> Value {
        // 1 Construct the JSON object.
        let mut obj = Map::new();

        // 2 Insert the key.
        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.key)),
        );

        // 3 Insert is registered? True.
        obj.insert("is_registered".to_string(), Value::Bool(true));

        // 4 Insert the registery index.
        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        // 5 Insert the rank.
        obj.insert(
            "rank".to_string(),
            match &self.rank {
                Some(rank) => Value::String(rank.to_string()),
                None => Value::Null,
            },
        );

        // 6 Insert the BLS key.
        obj.insert(
            "bls_key".to_string(),
            match &self.bls_key {
                Some(bls_key) => Value::String(hex::encode(bls_key)),
                None => Value::Null,
            },
        );

        // 7 Insert the secondary aggregation key.
        obj.insert(
            "secondary_aggregation_key".to_string(),
            match &self.secondary_aggregation_key {
                Some(secondary_aggregation_key) => {
                    Value::String(hex::encode(secondary_aggregation_key))
                }
                None => Value::Null,
            },
        );

        // 8 Insert the flame config.
        obj.insert(
            "flame_config".to_string(),
            match &self.flame_config {
                Some(flame_config) => flame_config.json(),
                None => Value::Null,
            },
        );

        // 9 Return the JSON object.
        Value::Object(obj)
    }
}

/// Helper function to serialize Option<[u8; 48]> as a byte vector.
fn serialize_bls_key<S>(bls_key: &Option<BLSKey>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match bls_key {
        Some(key) => key.as_slice().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// Helper function to deserialize Option<[u8; 48]> from a byte vector.
fn deserialize_bls_key<'de, D>(deserializer: D) -> Result<Option<BLSKey>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_bytes: Option<Vec<u8>> = Option::deserialize(deserializer)?;
    match opt_bytes {
        Some(bytes) => {
            if bytes.len() == 48 {
                let mut array = [0u8; 48];
                array.copy_from_slice(&bytes);
                Ok(Some(array))
            } else {
                Err(serde::de::Error::custom(format!(
                    "BLS key must be exactly 48 bytes, got {} bytes",
                    bytes.len()
                )))
            }
        }
        None => Ok(None),
    }
}

impl PartialEq for RegisteredAccount {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for RegisteredAccount {}

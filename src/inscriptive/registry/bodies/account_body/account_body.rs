use serde_json::{Map, Value};
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type AccountSecondaryAggregationKey = Vec<u8>;

/// Projector config key of an account.
type AccountProjectorConfig = [u8; 32];

// A struct for containing the registry index and call counter of an account.
#[derive(Clone)]
pub struct RMAccountBody {
    // Assigned registry index of an account.
    pub registry_index: u64,

    // Ever-increasing call counter of an account. 
    pub call_counter: u64,

    // Last observed activity timestamp of an account.
    pub last_activity_timestamp: u64,

    // BLS key of an account.
    pub primary_bls_key: Option<AccountBLSKey>,

    // Secondary aggregation key of an account.
    pub secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,

    // Projector config of an account.
    pub projector_config: Option<AccountProjectorConfig>,

    // Flame config of an account.
    pub flame_config: Option<FMAccountFlameConfig>,
}

impl RMAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(
        registry_index: u64,
        call_counter: u64,
        last_activity_timestamp: u64,
        primary_bls_key: Option<AccountBLSKey>,
        secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,
        projector_config: Option<AccountProjectorConfig>,
        flame_config: Option<FMAccountFlameConfig>,
    ) -> Self {
        Self {
            registry_index,
            call_counter,
            last_activity_timestamp,
            primary_bls_key,
            secondary_aggregation_key,
            projector_config,
            flame_config,
        }
    }

    /// Returns the account body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the account body JSON object.
        let mut obj = Map::new();

        // 2 Insert the registry index.
        obj.insert(
            "registry_index".to_string(),
            Value::String(self.registry_index.to_string()),
        );

        // 3 Insert the call counter.
        obj.insert(
            "call_counter".to_string(),
            Value::String(self.call_counter.to_string()),
        );

        // 4 Insert the last activity timestamp.
        obj.insert(
            "last_activity_timestamp".to_string(),
            Value::String(self.last_activity_timestamp.to_string()),
        );

        // 5 Insert the primary BLS key.
        obj.insert(
            "primary_bls_key".to_string(),
            match &self.primary_bls_key {
                Some(key) => Value::String(hex::encode(key)),
                None => Value::Null,
            },
        );

        // 6 Insert the secondary aggregation key.
        obj.insert(
            "secondary_aggregation_key".to_string(),
            match &self.secondary_aggregation_key {
                Some(key) => Value::String(hex::encode(key)),
                None => Value::Null,
            },
        );

        // 7 Insert the projector config.
        obj.insert(
            "projector_config".to_string(),
            match &self.projector_config {
                Some(config) => Value::String(hex::encode(config)),
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

        // 9 Return the account body JSON object.
        Value::Object(obj)
    }
}

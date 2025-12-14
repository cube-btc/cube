use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use serde_json::{Map, Value};

/// BLS key of an account.
type AccountBLSKey = [u8; 48];

/// Secondary aggregation key of an account (in case needed for post-quantum security).
type AccountSecondaryAggregationKey = Vec<u8>;

// A struct for containing the registery index and call counter of an account.
#[derive(Clone)]
pub struct RMAccountBody {
    // Assigned registery index of an account.
    pub registery_index: u64,

    // Ever-increasing call counter of an account.
    pub call_counter: u64,

    // BLS key of an account.
    pub primary_bls_key: Option<AccountBLSKey>,

    // Secondary aggregation key of an account.
    pub secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,

    // Flame config of an account.
    pub flame_config: Option<FMAccountFlameConfig>,
}

impl RMAccountBody {
    /// Constructs a fresh new account body.
    pub fn new(
        registery_index: u64,
        call_counter: u64,
        primary_bls_key: Option<AccountBLSKey>,
        secondary_aggregation_key: Option<AccountSecondaryAggregationKey>,
        flame_config: Option<FMAccountFlameConfig>,
    ) -> Self {
        Self {
            registery_index,
            call_counter,
            primary_bls_key,
            secondary_aggregation_key,
            flame_config,
        }
    }

    /// Returns the account body as a JSON object.
    pub fn json(&self) -> Value {
        // 1 Construct the account body JSON object.
        let mut obj = Map::new();

        // 2 Insert the registery index.
        obj.insert(
            "registery_index".to_string(),
            Value::String(self.registery_index.to_string()),
        );

        // 3 Insert the call counter.
        obj.insert(
            "call_counter".to_string(),
            Value::String(self.call_counter.to_string()),
        );

        // 4 Insert the primary BLS key.
        obj.insert(
            "primary_bls_key".to_string(),
            match &self.primary_bls_key {
                Some(key) => Value::String(hex::encode(key)),
                None => Value::Null,
            },
        );

        // 5 Insert the secondary aggregation key.
        obj.insert(
            "secondary_aggregation_key".to_string(),
            match &self.secondary_aggregation_key {
                Some(key) => Value::String(hex::encode(key)),
                None => Value::Null,
            },
        );

        // 6 Insert the flame config.
        obj.insert(
            "flame_config".to_string(),
            match &self.flame_config {
                Some(config) => config.json(),
                None => Value::Null,
            },
        );

        // 2 Return the account body JSON object.
        Value::Object(obj)
    }
}

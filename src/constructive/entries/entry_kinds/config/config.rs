use crate::constructive::core_types::entities::account::root_account::root_account::RootAccount;
use crate::constructive::core_types::target::target::Target;
use crate::inscriptive::flame_manager::flame_config::flame_config::FMAccountFlameConfig;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// `Config` is an `Entry` kind for mutating account configuration flags.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// The root account whose configuration is updated.
    pub root_account: RootAccount,
    /// Optional secondary aggregation key update.
    pub secondary_aggregation_key: Option<Vec<u8>>,
    /// Optional projector config update.
    pub projector_config: Option<[u8; 32]>,
    /// Optional flame config update.
    pub flame_config: Option<FMAccountFlameConfig>,
    /// Target execution information.
    pub target: Target,
}

impl Config {
    /// Creates a new `Config` entry kind.
    pub fn new(
        root_account: RootAccount,
        secondary_aggregation_key: Option<Vec<u8>>,
        projector_config: Option<[u8; 32]>,
        flame_config: Option<FMAccountFlameConfig>,
        target: Target,
    ) -> Self {
        Self {
            root_account,
            secondary_aggregation_key,
            projector_config,
            flame_config,
            target,
        }
    }

    /// Returns the config entry as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("entry_kind".to_string(), Value::String("config".to_string()));
        obj.insert("root_account".to_string(), self.root_account.json());
        obj.insert(
            "secondary_aggregation_key".to_string(),
            match &self.secondary_aggregation_key {
                Some(key) => Value::String(hex::encode(key)),
                None => Value::Null,
            },
        );
        obj.insert(
            "projector_config".to_string(),
            match &self.projector_config {
                Some(config) => Value::String(hex::encode(config)),
                None => Value::Null,
            },
        );
        obj.insert(
            "flame_config".to_string(),
            match &self.flame_config {
                Some(config) => config.json(),
                None => Value::Null,
            },
        );
        obj.insert(
            "target".to_string(),
            Value::Number(self.target.targeted_at_batch_height.into()),
        );
        Value::Object(obj)
    }
}

//! Config TCP response payload (bincode body).

use crate::constructive::entry::entry::entry::Entry;
use crate::operative::tasks::engine_session::session_pool::error::exec_config_in_pool_error::ExecConfigInPoolError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSuccessBody {
    pub entry_id: [u8; 32],
    pub batch_height: u64,
    pub batch_timestamp: u64,
    pub entry: Entry,
}

impl ConfigSuccessBody {
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "entry_id".to_string(),
            Value::String(hex::encode(self.entry_id)),
        );
        obj.insert(
            "batch_height".to_string(),
            Value::Number(self.batch_height.into()),
        );
        obj.insert(
            "batch_timestamp".to_string(),
            Value::Number(self.batch_timestamp.into()),
        );
        obj.insert("entry".to_string(), self.entry.json());
        Value::Object(obj)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ConfigResponseError {
    DeserializeConfigRequestError,
    ExecConfigInPoolError(ExecConfigInPoolError),
}

impl ConfigResponseError {
    pub fn json(&self) -> Value {
        match self {
            ConfigResponseError::DeserializeConfigRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_config_request_error".to_string()),
                );
                Value::Object(obj)
            }
            ConfigResponseError::ExecConfigInPoolError(e) => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("exec_config_in_pool_error".to_string()),
                );
                obj.insert(
                    "error".to_string(),
                    serde_json::to_value(e).unwrap_or_else(|_| Value::String(format!("{e:?}"))),
                );
                Value::Object(obj)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigResponseBody {
    Ok(ConfigSuccessBody),
    Err(ConfigResponseError),
}

impl ConfigResponseBody {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(r, _)| r)
    }

    pub fn json(&self) -> Value {
        match self {
            ConfigResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            ConfigResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn ok(entry_id: [u8; 32], batch_height: u64, batch_timestamp: u64, entry: Entry) -> Self {
        Self::Ok(ConfigSuccessBody {
            entry_id,
            batch_height,
            batch_timestamp,
            entry,
        })
    }

    pub fn err(e: ConfigResponseError) -> Self {
        Self::Err(e)
    }
}

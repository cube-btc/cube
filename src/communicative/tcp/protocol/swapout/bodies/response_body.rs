//! Swapout TCP response payload (bincode body).

use crate::constructive::entry::entry::entry::Entry;
use crate::operative::tasks::engine_session::session_pool::error::exec_swapout_in_pool_error::ExecSwapoutInPoolError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapoutSuccessBody {
    pub entry_id: [u8; 32],
    pub batch_height: u64,
    pub batch_timestamp: u64,
    pub entry: Entry,
}

impl SwapoutSuccessBody {
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
pub enum SwapoutResponseError {
    DeserializeSwapoutRequestError,
    ExecSwapoutInPoolError(ExecSwapoutInPoolError),
}

impl SwapoutResponseError {
    pub fn json(&self) -> Value {
        match self {
            SwapoutResponseError::DeserializeSwapoutRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_swapout_request_error".to_string()),
                );
                Value::Object(obj)
            }
            SwapoutResponseError::ExecSwapoutInPoolError(e) => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("exec_swapout_in_pool_error".to_string()),
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
pub enum SwapoutResponseBody {
    Ok(SwapoutSuccessBody),
    Err(SwapoutResponseError),
}

impl SwapoutResponseBody {
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
            SwapoutResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            SwapoutResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn ok(
        entry_id: [u8; 32],
        batch_height: u64,
        batch_timestamp: u64,
        entry: Entry,
    ) -> Self {
        Self::Ok(SwapoutSuccessBody {
            entry_id,
            batch_height,
            batch_timestamp,
            entry,
        })
    }

    pub fn err(e: SwapoutResponseError) -> Self {
        Self::Err(e)
    }
}

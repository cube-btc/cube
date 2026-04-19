//! Liftup v1 TCP response payload (bincode body).

use crate::constructive::entry::entry::entry::Entry;
use crate::operative::tasks::engine_session::session_pool::error::exec_liftup_in_pool_error::ExecLiftupInPoolError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiftupV1SuccessBody {
    pub entry_id: [u8; 32],
    pub batch_height: u64,
    pub batch_timestamp: u64,
    pub entry: Entry,
}

impl LiftupV1SuccessBody {
    /// JSON object using [`Entry::json`](Entry::json) for the embedded entry (not Serde’s `Entry` shape).
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

/// Failure cases for a Liftup v1 response body.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LiftupV1ResponseError {
    DeserializeLiftupV1RequestError,
    ExecLiftupInPoolError(ExecLiftupInPoolError),
}

impl LiftupV1ResponseError {
    pub fn json(&self) -> Value {
        match self {
            LiftupV1ResponseError::DeserializeLiftupV1RequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_liftup_v1_request_error".to_string()),
                );
                Value::Object(obj)
            }
            LiftupV1ResponseError::ExecLiftupInPoolError(e) => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("exec_liftup_in_pool_error".to_string()),
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
pub enum LiftupV1ResponseBody {
    Ok(LiftupV1SuccessBody),
    Err(LiftupV1ResponseError),
}

impl LiftupV1ResponseBody {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(r, _)| r)
    }

    /// JSON object: success uses [`LiftupV1SuccessBody::json`], errors use [`LiftupV1ResponseError::json`].
    pub fn json(&self) -> Value {
        match self {
            LiftupV1ResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            LiftupV1ResponseBody::Err(e) => {
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
        Self::Ok(LiftupV1SuccessBody {
            entry_id,
            batch_height,
            batch_timestamp,
            entry,
        })
    }

    pub fn err(e: LiftupV1ResponseError) -> Self {
        Self::Err(e)
    }
}

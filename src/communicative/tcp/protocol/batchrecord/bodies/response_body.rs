//! Batch record TCP response payload (bincode body).

use crate::constructive::bitcoiny::batch_record::batch_record::BatchRecord;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct BatchRecordSuccessBody {
    pub batch_record: Option<BatchRecord>,
}

impl BatchRecordSuccessBody {
    /// JSON object using [`BatchRecord::json`](BatchRecord::json) when present.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "batch_record".to_string(),
            match &self.batch_record {
                Some(record) => record.json(),
                None => Value::Null,
            },
        );
        Value::Object(obj)
    }
}

/// Failure cases for a Batch record response body.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BatchRecordResponseError {
    DeserializeBatchRecordRequestError,
    ArchivalManagerUnavailableError,
}

impl BatchRecordResponseError {
    pub fn json(&self) -> Value {
        match self {
            BatchRecordResponseError::DeserializeBatchRecordRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_batch_record_request_error".to_string()),
                );
                Value::Object(obj)
            }
            BatchRecordResponseError::ArchivalManagerUnavailableError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("archival_manager_unavailable_error".to_string()),
                );
                Value::Object(obj)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum BatchRecordResponseBody {
    Ok(BatchRecordSuccessBody),
    Err(BatchRecordResponseError),
}

impl BatchRecordResponseBody {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(r, _)| r)
    }

    /// JSON object: success uses [`BatchRecordSuccessBody::json`], errors use [`BatchRecordResponseError::json`].
    pub fn json(&self) -> Value {
        match self {
            BatchRecordResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            BatchRecordResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn ok(batch_record: Option<BatchRecord>) -> Self {
        Self::Ok(BatchRecordSuccessBody { batch_record })
    }

    pub fn err(e: BatchRecordResponseError) -> Self {
        Self::Err(e)
    }
}

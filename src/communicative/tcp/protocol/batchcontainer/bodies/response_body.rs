//! Batch container TCP response payload (bincode body).

use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct BatchContainerSuccessBody {
    pub batch_container: Option<BatchContainer>,
}

impl BatchContainerSuccessBody {
    /// JSON object using [`BatchContainer::json`](BatchContainer::json) when present.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "batch_container".to_string(),
            match &self.batch_container {
                Some(batch_container) => batch_container.json(),
                None => Value::Null,
            },
        );
        Value::Object(obj)
    }
}

/// Failure cases for a Batch container response body.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BatchContainerResponseError {
    DeserializeBatchContainerRequestError,
    ArchivalManagerUnavailableError,
}

impl BatchContainerResponseError {
    pub fn json(&self) -> Value {
        match self {
            BatchContainerResponseError::DeserializeBatchContainerRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_batch_container_request_error".to_string()),
                );
                Value::Object(obj)
            }
            BatchContainerResponseError::ArchivalManagerUnavailableError => {
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
pub enum BatchContainerResponseBody {
    Ok(BatchContainerSuccessBody),
    Err(BatchContainerResponseError),
}

impl BatchContainerResponseBody {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(r, _)| r)
    }

    /// JSON object: success uses [`BatchContainerSuccessBody::json`], errors use [`BatchContainerResponseError::json`].
    pub fn json(&self) -> Value {
        match self {
            BatchContainerResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            BatchContainerResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn ok(batch_container: Option<BatchContainer>) -> Self {
        Self::Ok(BatchContainerSuccessBody { batch_container })
    }

    pub fn err(e: BatchContainerResponseError) -> Self {
        Self::Err(e)
    }
}

//! In-flight sync TCP response payload (bincode body).

use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Failure cases for an In-flight sync response body.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum InFlightSyncResponseError {
    DeserializeInFlightSyncRequestError,
    ArchivalManagerUnavailableError,
    BatchContainerNotFoundError,
}

impl InFlightSyncResponseError {
    pub fn json(&self) -> Value {
        match self {
            InFlightSyncResponseError::DeserializeInFlightSyncRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("deserialize_in_flight_sync_request_error".to_string()),
                );
                Value::Object(obj)
            }
            InFlightSyncResponseError::ArchivalManagerUnavailableError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("archival_manager_unavailable_error".to_string()),
                );
                Value::Object(obj)
            }
            InFlightSyncResponseError::BatchContainerNotFoundError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String("batch_container_not_found_error".to_string()),
                );
                Value::Object(obj)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InFlightSyncResponseBody {
    FullySynced,
    BatchDownload(BatchContainer),
    Err(InFlightSyncResponseError),
}

impl InFlightSyncResponseBody {
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
            InFlightSyncResponseBody::FullySynced => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert(
                    "result".to_string(),
                    Value::String("fully_synced".to_string()),
                );
                Value::Object(obj)
            }
            InFlightSyncResponseBody::BatchDownload(batch_container) => {
                let mut result = Map::new();
                result.insert("kind".to_string(), Value::String("batch_download".to_string()));
                result.insert("batch_container".to_string(), batch_container.json());

                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), Value::Object(result));
                Value::Object(obj)
            }
            InFlightSyncResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn fully_synced() -> Self {
        Self::FullySynced
    }

    pub fn batch_download(batch_container: BatchContainer) -> Self {
        Self::BatchDownload(batch_container)
    }

    pub fn err(e: InFlightSyncResponseError) -> Self {
        Self::Err(e)
    }
}

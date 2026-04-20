//! Batch container-by-prevoutpoint TCP response payload (bincode body).

use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct BatchContainerByPrevOutpointSuccessBody {
    pub batch_container: Option<BatchContainer>,
}

impl BatchContainerByPrevOutpointSuccessBody {
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BatchContainerByPrevOutpointResponseError {
    DeserializeBatchContainerByPrevOutpointRequestError,
    ArchivalManagerUnavailableError,
}

impl BatchContainerByPrevOutpointResponseError {
    pub fn json(&self) -> Value {
        match self {
            BatchContainerByPrevOutpointResponseError::DeserializeBatchContainerByPrevOutpointRequestError => {
                let mut obj = Map::new();
                obj.insert(
                    "kind".to_string(),
                    Value::String(
                        "deserialize_batch_container_by_prev_outpoint_request_error".to_string(),
                    ),
                );
                Value::Object(obj)
            }
            BatchContainerByPrevOutpointResponseError::ArchivalManagerUnavailableError => {
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
pub enum BatchContainerByPrevOutpointResponseBody {
    Ok(BatchContainerByPrevOutpointSuccessBody),
    Err(BatchContainerByPrevOutpointResponseError),
}

impl BatchContainerByPrevOutpointResponseBody {
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
            BatchContainerByPrevOutpointResponseBody::Ok(body) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("ok".to_string()));
                obj.insert("result".to_string(), body.json());
                Value::Object(obj)
            }
            BatchContainerByPrevOutpointResponseBody::Err(e) => {
                let mut obj = Map::new();
                obj.insert("status".to_string(), Value::String("err".to_string()));
                obj.insert("error".to_string(), e.json());
                Value::Object(obj)
            }
        }
    }

    pub fn ok(batch_container: Option<BatchContainer>) -> Self {
        Self::Ok(BatchContainerByPrevOutpointSuccessBody { batch_container })
    }

    pub fn err(e: BatchContainerByPrevOutpointResponseError) -> Self {
        Self::Err(e)
    }
}

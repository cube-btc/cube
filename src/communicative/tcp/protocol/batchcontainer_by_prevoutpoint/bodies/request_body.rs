//! Batch container-by-prevoutpoint TCP request payload (bincode body).

use bitcoin::OutPoint;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchContainerByPrevOutpointRequestBody {
    pub prev_payload_outpoint: OutPoint,
}

impl BatchContainerByPrevOutpointRequestBody {
    pub fn new(prev_payload_outpoint: OutPoint) -> Self {
        Self {
            prev_payload_outpoint,
        }
    }

    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(req, _)| req)
    }
}

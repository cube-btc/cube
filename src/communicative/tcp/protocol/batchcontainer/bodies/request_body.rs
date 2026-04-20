//! Batch container TCP request payload (bincode body).

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchContainerRequestBody {
    pub batch_height: u64,
}

impl BatchContainerRequestBody {
    pub fn new(batch_height: u64) -> Self {
        Self { batch_height }
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

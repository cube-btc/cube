//! In-flight sync TCP request payload (bincode body).

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InFlightSyncRequestBody {
    pub cube_batch_sync_height_tip: u64,
}

impl InFlightSyncRequestBody {
    pub fn new(cube_batch_sync_height_tip: u64) -> Self {
        Self {
            cube_batch_sync_height_tip,
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

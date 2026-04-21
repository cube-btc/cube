//! Move TCP request payload (bincode body).

use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use serde::{Deserialize, Serialize};

mod bls_signature_96 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &[u8; 96], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (a, rest) = bytes.split_at(32);
        let (b, c) = rest.split_at(32);
        let parts = (
            <[u8; 32]>::try_from(a).expect("split_at(32)"),
            <[u8; 32]>::try_from(b).expect("split_at(32)"),
            <[u8; 32]>::try_from(c).expect("split_at(32)"),
        );
        parts.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 96], D::Error>
    where
        D: Deserializer<'de>,
    {
        let (a, b, c) = <([u8; 32], [u8; 32], [u8; 32])>::deserialize(deserializer)?;
        let mut out = [0u8; 96];
        out[0..32].copy_from_slice(&a);
        out[32..64].copy_from_slice(&b);
        out[64..96].copy_from_slice(&c);
        Ok(out)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRequestBody {
    pub move_entry: Move,
    #[serde(with = "bls_signature_96")]
    pub move_bls_signature: [u8; 96],
}

impl MoveRequestBody {
    pub fn new(move_entry: Move, move_bls_signature: [u8; 96]) -> Self {
        Self {
            move_entry,
            move_bls_signature,
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

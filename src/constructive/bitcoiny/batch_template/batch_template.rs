use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use bit_vec::BitVec;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A struct that represents a batch template.
#[derive(Clone, Serialize, Deserialize)]
pub struct BatchTemplate {
    /// The Bitcoin transaction inputs of the batch.
    pub entries: Vec<Entry>,

    /// The payload of the batch in bytes.
    pub payload: Payload,

    /// The projector of the batch.
    pub projector: Option<Projector>,
}

impl BatchTemplate {
    /// Creates a new batch template.
    pub fn new(entries: Vec<Entry>, payload: Payload, projector: Option<Projector>) -> Self {
        Self {
            entries,
            payload,
            projector,
        }
    }

    /// Returns the payload of the batch in bits.
    pub fn payload_bits(&self) -> Option<BitVec> {
        BitVec::from_ape_payload_bytes(self.payload.payload_bytes.clone())
    }

    /// Returns the batch template as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "payload_bytes".to_string(),
            Value::String(hex::encode(&self.payload.payload_bytes)),
        );

        Value::Object(obj)
    }
}

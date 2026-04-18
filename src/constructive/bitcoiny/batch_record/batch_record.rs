use crate::constructive::bitcoiny::batch_container::batch_container::BatchContainer;
use crate::constructive::txn::ext::OutpointExt;
use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::txout_types::payload::payload::Payload;
use crate::constructive::txout_types::projector::projector::Projector;
use crate::transmutative::bls::bls_ser::{deserialize_bls_signature, serialize_bls_signature};
use bitcoin::OutPoint;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents a batch record.
#[derive(Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    pub batch_container: BatchContainer,
    pub batch_timestamp: u64,
    pub payload_version: u32,
    #[serde(
        serialize_with = "serialize_bls_signature",
        deserialize_with = "deserialize_bls_signature"
    )]
    pub aggregate_bls_signature: [u8; 96],
    pub executed_entries: Vec<Entry>,
    pub expired_projector_outpoints: Vec<OutPoint>,
    pub new_payload: Payload,
    pub new_projector: Option<Projector>,
}

impl BatchRecord {
    /// Constructs a batch record.
    pub fn new(
        batch_container: BatchContainer,
        batch_timestamp: u64,
        payload_version: u32,
        aggregate_bls_signature: [u8; 96],
        executed_entries: Vec<Entry>,
        expired_projector_outpoints: Vec<OutPoint>,
        new_payload: Payload,
        new_projector: Option<Projector>,
    ) -> Self {
        Self {
            batch_container,
            batch_timestamp,
            payload_version,
            aggregate_bls_signature,
            executed_entries,
            expired_projector_outpoints,
            new_payload,
            new_projector,
        }
    }

    /// Serializes this value with bincode.
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes a batch record from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(batch_record, _)| batch_record)
    }

    /// Returns this batch record as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "batch_height".to_string(),
            Value::Number(self.batch_container.batch_height.into()),
        );

        obj.insert(
            "batch_timestamp".to_string(),
            Value::Number(self.batch_timestamp.into()),
        );

        obj.insert(
            "batch_txid".to_string(),
            Value::String(self.batch_container.signed_batch_txn.txid().to_string()),
        );

        obj.insert(
            "payload_version".to_string(),
            Value::Number(self.payload_version.into()),
        );

        obj.insert(
            "aggregate_bls_signature".to_string(),
            Value::String(hex::encode(self.aggregate_bls_signature)),
        );

        obj.insert(
            "executed_entries".to_string(),
            Value::Array(
                self.executed_entries
                    .iter()
                    .map(Entry::json)
                    .collect(),
            ),
        );

        obj.insert(
            "expired_projector_outpoints".to_string(),
            Value::Array(
                self.expired_projector_outpoints
                    .iter()
                    .map(|op| json_outpoint(op))
                    .collect(),
            ),
        );

        obj.insert("new_payload".to_string(), self.new_payload.json());

        obj.insert(
            "new_projector".to_string(),
            match self.new_projector.as_ref() {
                Some(projector) => projector.json(),
                None => Value::Null,
            },
        );

        Value::Object(obj)
    }
}

fn json_outpoint(outpoint: &OutPoint) -> Value {
    let mut o = Map::new();
    o.insert(
        "txid".to_string(),
        Value::String(hex::encode(outpoint.txhash())),
    );
    o.insert("vout".to_string(), Value::Number(outpoint.vout.into()));
    Value::Object(o)
}

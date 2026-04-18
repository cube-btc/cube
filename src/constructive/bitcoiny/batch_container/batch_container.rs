use crate::constructive::bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn;
use bitcoin::{OutPoint, TxOut, hashes::Hash as _};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents a batch container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchContainer {
    /// Batch height.
    pub batch_height: u64,

    /// New payload bytes.
    pub new_payload_bytes: Vec<u8>,

    /// Signed batch transaction.
    pub signed_batch_txn: SignedBatchTxn,
}

impl BatchContainer {
    /// Constructs a batch container.
    pub fn new(
        batch_height: u64,
        new_payload_bytes: Vec<u8>,
        signed_batch_txn: SignedBatchTxn,
    ) -> Self {
        Self {
            batch_height,
            new_payload_bytes,
            signed_batch_txn,
        }
    }

    /// Returns the batch height.
    pub fn batch_height(&self) -> u64 {
        self.batch_height
    }

    /// Returns the batch txid.
    pub fn batch_txid(&self) -> [u8; 32] {
        self.signed_batch_txn.txid().to_byte_array()
    }

    /// Returns the new payload bytes.
    pub fn new_payload_bytes(&self) -> Vec<u8> {
        self.new_payload_bytes.clone()
    }

    /// Returns the transaction input outpoints.
    pub fn bitcoin_tx_inputs(&self) -> Vec<OutPoint> {
        self.signed_batch_txn.tx_input_outpoints()
    }

    /// Returns the Bitcoin transaction outputs.
    pub fn bitcoin_tx_outputs(&self) -> Vec<TxOut> {
        self.signed_batch_txn.tx_outputs()
    }

    /// Returns this batch container as a JSON object (includes `txid`).
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "batch_height".to_string(),
            Value::Number(self.batch_height.into()),
        );

        obj.insert(
            "txid".to_string(),
            Value::String(self.signed_batch_txn.txid().to_string()),
        );

        obj.insert(
            "new_payload_bytes".to_string(),
            Value::String(hex::encode(&self.new_payload_bytes)),
        );

        obj.insert(
            "signed_batch_txn".to_string(),
            serde_json::to_value(&self.signed_batch_txn)
                .expect("SignedBatchTxn must serialize to JSON"),
        );

        Value::Object(obj)
    }

    /// Serializes this value with bincode.
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serde::encode_to_vec(self, bincode::config::standard()).ok()
    }

    /// Deserializes a batch container from bincode bytes.
    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        bincode::serde::decode_from_slice::<Self, _>(bytes, bincode::config::standard())
            .ok()
            .map(|(batch_container, _)| batch_container)
    }
}

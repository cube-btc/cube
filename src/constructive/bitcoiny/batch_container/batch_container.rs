use crate::constructive::{
    bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn,
    txout_types::payload::payload::Payload,
};
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};

/// Represents a batch container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchContainer {
    /// Batch height.
    pub batch_height: u64,

    /// Engine x-only public key (identifies the payload taproot).
    pub engine_key: [u8; 32],

    /// Payload bytes.
    pub payload_bytes: Vec<u8>,

    /// Signed batch transaction.
    pub signed_batch_txn: SignedBatchTxn,
}

impl BatchContainer {
    /// Constructs a batch container.
    pub fn new(
        batch_height: u64,
        engine_key: [u8; 32],
        payload_bytes: Vec<u8>,
        signed_batch_txn: SignedBatchTxn,
    ) -> Self {
        Self {
            batch_height,
            engine_key,
            payload_bytes,
            signed_batch_txn,
        }
    }

    /// Returns the batch height.
    pub fn batch_height(&self) -> u64 {
        self.batch_height
    }

    /// Returns the payload bytes.
    pub fn payload_bytes(&self) -> Vec<u8> {
        self.payload_bytes.clone()
    }

    /// Returns the transaction input outpoints.
    pub fn bitcoin_tx_inputs(&self) -> Vec<OutPoint> {
        self.signed_batch_txn.tx_input_outpoints()
    }

    /// Returns the Bitcoin transaction outputs.
    pub fn bitcoin_tx_outputs(&self) -> Vec<TxOut> {
        self.signed_batch_txn.tx_outputs()
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

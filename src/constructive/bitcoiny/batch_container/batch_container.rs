use crate::constructive::bitcoiny::batch_txn::signed_batch_txn::signed_batch_txn::SignedBatchTxn;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};

/// Represents a batch container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchContainer {
    /// Batch height.
    pub batch_height: u64,

    /// Payload bytes.
    pub payload_bytes: Vec<u8>,

    /// Signed batch transaction.
    pub signed_batch_txn: SignedBatchTxn,
}

impl BatchContainer {
    /// Constructs a batch container.
    pub fn new(
        batch_height: u64,
        payload_bytes: Vec<u8>,
        signed_batch_txn: SignedBatchTxn,
    ) -> Self {
        Self {
            batch_height,
            payload_bytes,
            signed_batch_txn,
        }
    }

    /// Returns the transaction input outpoints.
    pub fn bitcoin_tx_inputs(&self) -> Vec<OutPoint> {
        self.signed_batch_txn.tx_input_outpoints()
    }

    /// Returns the Bitcoin transaction outputs.
    pub fn bitcoin_tx_outputs(&self) -> Vec<TxOut> {
        self.signed_batch_txn.tx_outputs()
    }
}

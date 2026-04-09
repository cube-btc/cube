use crate::transmutative::codec::bitvec_ext::BitVecExt;
use bit_vec::BitVec;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

/// A struct that represents a batch template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchTemplate {
    /// The Bitcoin transaction inputs of the batch.
    pub bitcoin_tx_inputs: Vec<OutPoint>,

    /// The Bitcoin transaction outputs of the batch.
    pub bitcoin_tx_outputs: Vec<TxOut>,

    /// The payload of the batch in bytes.
    pub payload_bytes: Bytes,
}

impl BatchTemplate {
    /// Creates a new batch template.
    pub fn new(
        bitcoin_tx_inputs: Vec<OutPoint>,
        bitcoin_tx_outputs: Vec<TxOut>,
        payload_bytes: Bytes,
    ) -> Self {
        Self {
            bitcoin_tx_inputs,
            bitcoin_tx_outputs,
            payload_bytes,
        }
    }

    /// Returns the payload of the batch in bits.
    pub fn payload_bits(&self) -> Option<BitVec> {
        BitVec::from_payload_bytes(self.payload_bytes.clone())
    }
}

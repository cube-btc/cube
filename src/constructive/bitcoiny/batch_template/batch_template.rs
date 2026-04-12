use crate::constructive::txn::ext::OutpointExt;
use crate::transmutative::codec::bitvec_ext::BitVecExt;
use bit_vec::BitVec;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
        BitVec::from_ape_payload_bytes(self.payload_bytes.clone())
    }

    /// Returns the batch template as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "bitcoin_tx_inputs".to_string(),
            Value::Array(
                self.bitcoin_tx_inputs
                    .iter()
                    .map(|outpoint| {
                        let mut o = Map::new();
                        o.insert(
                            "txid".to_string(),
                            Value::String(hex::encode(outpoint.txhash())),
                        );
                        o.insert(
                            "vout".to_string(),
                            Value::Number(outpoint.vout().into()),
                        );
                        Value::Object(o)
                    })
                    .collect(),
            ),
        );

        obj.insert(
            "bitcoin_tx_outputs".to_string(),
            Value::Array(
                self.bitcoin_tx_outputs
                    .iter()
                    .map(|txout| {
                        let mut o = Map::new();
                        o.insert(
                            "satoshis".to_string(),
                            Value::Number(txout.value.to_sat().into()),
                        );
                        o.insert(
                            "scriptpubkey".to_string(),
                            Value::String(hex::encode(txout.script_pubkey.as_bytes())),
                        );
                        Value::Object(o)
                    })
                    .collect(),
            ),
        );

        obj.insert(
            "payload_bytes".to_string(),
            Value::String(hex::encode(&self.payload_bytes)),
        );

        Value::Object(obj)
    }
}

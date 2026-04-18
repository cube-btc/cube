use crate::constructive::txn::ext::OutpointExt;
use bitcoin::{OutPoint, TxOut};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct Projector {
    pub scriptpubkey: Vec<u8>,
    pub satoshi_amount: u64,
    pub location: Option<(OutPoint, TxOut)>,
}

impl Projector {
    /// Returns the location of the Projector.
    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        self.location.clone()
    }

    /// Returns this projector as a JSON object.
    pub fn json(&self) -> Value {
        let mut obj = Map::new();

        obj.insert(
            "scriptpubkey".to_string(),
            Value::String(hex::encode(&self.scriptpubkey)),
        );

        obj.insert(
            "satoshi_amount".to_string(),
            Value::Number(self.satoshi_amount.into()),
        );

        obj.insert(
            "location".to_string(),
            match self.location.as_ref() {
                Some(loc) => projector_location_json(loc),
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

fn json_txout(txout: &TxOut) -> Value {
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
}

fn projector_location_json((outpoint, txout): &(OutPoint, TxOut)) -> Value {
    let mut o = Map::new();
    o.insert("outpoint".to_string(), json_outpoint(outpoint));
    o.insert("txout".to_string(), json_txout(txout));
    Value::Object(o)
}
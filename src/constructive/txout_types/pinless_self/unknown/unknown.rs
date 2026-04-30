use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PinlessSelfUnknown {
    pub custom_scriptpubkey: Vec<u8>,
    pub location: Option<(OutPoint, TxOut)>,
}

impl PinlessSelfUnknown {
    pub fn new(custom_scriptpubkey: Vec<u8>, location: Option<(OutPoint, TxOut)>) -> Self {
        Self {
            custom_scriptpubkey,
            location,
        }
    }

    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        self.location.clone()
    }

    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), Value::String("unknown".to_string()));
        obj.insert(
            "custom_scriptpubkey".to_string(),
            Value::String(hex::encode(&self.custom_scriptpubkey)),
        );
        obj.insert(
            "location".to_string(),
            match self.location.as_ref() {
                Some((outpoint, txout)) => {
                    let mut o = Map::new();
                    let mut outpoint_json = Map::new();
                    outpoint_json.insert("txid".to_string(), Value::String(hex::encode(outpoint.txid)));
                    outpoint_json.insert("vout".to_string(), Value::Number(outpoint.vout.into()));
                    o.insert("outpoint".to_string(), Value::Object(outpoint_json));

                    let mut txout_json = Map::new();
                    txout_json.insert(
                        "satoshis".to_string(),
                        Value::Number(txout.value.to_sat().into()),
                    );
                    txout_json.insert(
                        "scriptpubkey".to_string(),
                        Value::String(hex::encode(txout.script_pubkey.as_bytes())),
                    );
                    o.insert("txout".to_string(), Value::Object(txout_json));
                    Value::Object(o)
                }
                None => Value::Null,
            },
        );
        Value::Object(obj)
    }
}

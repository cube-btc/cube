use super::default::PinlessSelfDefault;
use crate::constructive::txn::ext::OutpointExt;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

type Bytes = Vec<u8>;

/// Pinning attack-resistant self tx out type for `Swapout` entry kind.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PinlessSelf {
    // Default PinlessSelf: self account key can spend after one block.
    Default(PinlessSelfDefault),

    // Unknown PinlessSelf: scriptpubkey content is unknown, but trusted to not attempt a pinning attack.
    Unknown { location: Option<(OutPoint, TxOut)> },
}

impl PinlessSelf {
    pub fn new_default(account_key: [u8; 32], location: Option<(OutPoint, TxOut)>) -> PinlessSelf {
        PinlessSelf::Default(PinlessSelfDefault::new(account_key, location))
    }

    pub fn new_unknown(location: Option<(OutPoint, TxOut)>) -> PinlessSelf {
        PinlessSelf::Unknown { location }
    }

    pub fn account_key(&self) -> Option<[u8; 32]> {
        match self {
            PinlessSelf::Default(pinless_self_default) => Some(pinless_self_default.account_key),
            PinlessSelf::Unknown { .. } => None,
        }
    }

    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        match self {
            PinlessSelf::Default(pinless_self_default) => pinless_self_default.location(),
            PinlessSelf::Unknown { location } => location.clone(),
        }
    }

    pub fn outpoint(&self) -> Option<OutPoint> {
        self.location().as_ref().map(|(outpoint, _)| *outpoint)
    }

    pub fn txout(&self) -> Option<TxOut> {
        self.location().as_ref().map(|(_, txout)| txout.clone())
    }

    pub fn calculated_scriptpubkey(&self) -> Option<Bytes> {
        match self {
            PinlessSelf::Default(pinless_self_default) => {
                pinless_self_default.calculated_scriptpubkey()
            }
            PinlessSelf::Unknown { .. } => None,
        }
    }

    pub fn validate_scriptpubkey(&self) -> Option<bool> {
        match self {
            PinlessSelf::Default(pinless_self_default) => {
                Some(pinless_self_default.validate_scriptpubkey())
            }
            PinlessSelf::Unknown { .. } => None,
        }
    }

    pub fn json(&self) -> Value {
        match self {
            PinlessSelf::Default(pinless_self_default) => pinless_self_default.json(),
            PinlessSelf::Unknown { location } => {
                let mut obj = Map::new();
                obj.insert("version".to_string(), Value::String("unknown".to_string()));
                obj.insert(
                    "location".to_string(),
                    match location.as_ref() {
                        Some(loc) => pinless_self_location_json(loc),
                        None => Value::Null,
                    },
                );
                Value::Object(obj)
            }
        }
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

fn pinless_self_location_json((outpoint, txout): &(OutPoint, TxOut)) -> Value {
    let mut o = Map::new();
    o.insert("outpoint".to_string(), json_outpoint(outpoint));
    o.insert("txout".to_string(), json_txout(txout));
    Value::Object(o)
}

use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::constructive::txn::ext::OutpointExt;
use crate::transmutative::codec::csv::{CSVEncode, CSVFlag};
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

type Bytes = Vec<u8>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PinlessSelfDefault {
    pub account_key: [u8; 32],
    pub location: Option<(OutPoint, TxOut)>,
}

impl PinlessSelfDefault {
    pub fn new(account_key: [u8; 32], location: Option<(OutPoint, TxOut)>) -> PinlessSelfDefault {
        PinlessSelfDefault {
            account_key,
            location,
        }
    }

    pub fn location(&self) -> Option<(OutPoint, TxOut)> {
        self.location.clone()
    }

    pub fn outpoint(&self) -> Option<OutPoint> {
        self.location.as_ref().map(|(outpoint, _)| *outpoint)
    }

    pub fn txout(&self) -> Option<TxOut> {
        self.location.as_ref().map(|(_, txout)| txout.clone())
    }

    pub fn calculated_scriptpubkey(&self) -> Option<Bytes> {
        return_pinless_self_default_scriptpubkey(self.account_key)
    }

    pub fn validate_scriptpubkey(&self) -> bool {
        let self_txout = match self.txout() {
            Some(txout) => txout,
            None => return false,
        };

        let calculated_scriptpubkey = match self.calculated_scriptpubkey() {
            Some(scriptpubkey) => scriptpubkey,
            None => return false,
        };

        self_txout.script_pubkey.as_bytes() == calculated_scriptpubkey.as_slice()
    }

    pub fn json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("version".to_string(), Value::String("default".to_string()));
        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.account_key)),
        );
        obj.insert(
            "location".to_string(),
            match self.location.as_ref() {
                Some(loc) => pinless_self_location_json(loc),
                None => Value::Null,
            },
        );
        Value::Object(obj)
    }
}

pub fn return_pinless_self_default_tapscript(account_key: [u8; 32]) -> Option<Vec<u8>> {
    let mut tapscript = Vec::<u8>::new();

    // 2.5.1 <3 months>
    tapscript.extend(Vec::<u8>::csv_num_encode(CSVFlag::CSVBlock));

    // 2.5.2 OP_CHECKSEQUENCEVERIFY (0xb2)
    tapscript.push(0xb2);

    // 2.5.3 OP_DROP (0x75)
    tapscript.push(0x75);

    // 2.3.1 Push account key
    tapscript.push(0x20); // OP_PUSHDATA_32
    tapscript.extend(account_key); // Account Key 32-bytes

    // 2.3.2 OP_CHECKSIG (0xac)
    tapscript.push(0xac);

    Some(tapscript)
}

pub fn return_pinless_self_default_taproot(account_key: [u8; 32]) -> Option<TapRoot> {
    let tapscript = return_pinless_self_default_tapscript(account_key)?;
    let tapleaf = TapLeaf::new(tapscript);
    let taproot = TapRoot::script_path_only_single(tapleaf);
    Some(taproot)
}

pub fn return_pinless_self_default_scriptpubkey(account_key: [u8; 32]) -> Option<Bytes> {
    let taproot = return_pinless_self_default_taproot(account_key)?;
    taproot.spk()
}

impl P2TR for PinlessSelfDefault {
    fn taproot(&self) -> Option<TapRoot> {
        return_pinless_self_default_taproot(self.account_key)
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

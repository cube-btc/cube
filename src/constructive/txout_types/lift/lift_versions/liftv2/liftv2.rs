use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::constructive::txn::ext::OutpointExt;
use crate::transmutative::codec::csv::{CSVEncode, CSVFlag};
use crate::transmutative::musig::keyagg::MusigKeyAggCtx;
use crate::transmutative::secp::into::IntoPoint;
use bitcoin::{OutPoint, TxOut};
use hex;
use secp::Point;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// A type alias for bytes.
type Bytes = Vec<u8>;

/// MuSig2-based and thus interactive, but trustless Lift implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiftV2 {
    pub account_key: [u8; 32],
    pub engine_key: [u8; 32],
    pub outpoint: OutPoint,
    pub txout: TxOut,
}

impl LiftV2 {
    /// Creates a new LiftV2 struct.
    pub fn new(
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    ) -> LiftV2 {
        LiftV2 {
            account_key,
            engine_key,
            outpoint,
            txout,
        }
    }

    /// Validates the LiftV2 scriptpubkey.
    pub fn validate_scriptpubkey(&self) -> bool {
        // 1 Calculate the scriptpubkey.
        let calculated_scriptpubkey =
            match return_liftv2_scriptpubkey(self.account_key, self.engine_key) {
                Some(scriptpubkey) => scriptpubkey,
                None => return false,
            };

        // 2 Get the self scriptpubkey.
        let self_scriptpubkey = self.txout.script_pubkey.as_bytes();

        // 3 Validate the scriptpubkeys.
        if &calculated_scriptpubkey != self_scriptpubkey {
            return false;
        }

        // 4 Return true.
        true
    }

    /// Returns a JSON representation of the Lift struct
    pub fn json(&self) -> Value {
        // 1 Construct the lift JSON object
        let mut obj = Map::new();

        // 2 Add lift version tag
        obj.insert(
            "version".to_string(),
            Value::String("v2_interactive".to_string()),
        );

        // 3 Add account key
        obj.insert(
            "account_key".to_string(),
            Value::String(hex::encode(self.account_key)),
        );

        // 4 Add the Engine key
        obj.insert(
            "engine_key".to_string(),
            Value::String(hex::encode(self.engine_key)),
        );

        // 5 Construct outpoint object
        let outpoint_obj = {
            let mut obj = Map::new();

            obj.insert(
                "txid".to_string(),
                Value::String(hex::encode(self.outpoint.txhash())),
            );
            obj.insert("vout".to_string(), Value::Number(self.outpoint.vout.into()));

            obj
        };

        // 6 Add outpoint to the JSON object
        obj.insert("outpoint".to_string(), Value::Object(outpoint_obj));

        // 7 Construct txout object
        let txout_obj = {
            let mut obj = Map::new();

            obj.insert(
                "satoshis".to_string(),
                Value::Number(self.txout.value.to_sat().into()),
            );
            obj.insert(
                "scriptpubkey".to_string(),
                Value::String(hex::encode(self.txout.script_pubkey.as_bytes())),
            );

            obj
        };

        // 8 Add txout to the JSON object
        obj.insert("txout".to_string(), Value::Object(txout_obj));

        // 9 Return the lift JSON object
        Value::Object(obj)
    }
}

/// Returns a taproot for the LiftV2 struct.
pub fn return_liftv2_taproot(account_key: [u8; 32], engine_key: [u8; 32]) -> Option<TapRoot> {
    // 1 Construct the keys
    let keys: Vec<Point> = {
        let mut keys = Vec::<Point>::new();

        keys.push(account_key.into_point().unwrap());
        keys.push(engine_key.into_point().unwrap());

        keys
    };

    // 2 Construct the key aggregation context
    let key_agg_ctx = MusigKeyAggCtx::new(&keys, None)?;

    // 3 Construct the inner key
    let agg_inner_key = key_agg_ctx.agg_inner_key();

    // 4 Construct the sweep path
    let sweep_path_tapscript: Vec<u8> = {
        let mut tapscript = Vec::<u8>::new();

        tapscript.extend(Bytes::csv_script(CSVFlag::CSVThreeMonths)); // Relative Timelock
        tapscript.push(0x20); // OP_PUSHDATA_32
        tapscript.extend(account_key); // Account Key 32-bytes
        tapscript.push(0xac); // OP_CHECKSIG

        tapscript
    };

    // 5 Construct the sweep path leaf
    let sweep_path = TapLeaf::new(sweep_path_tapscript);

    // 6 Construct the taproot
    let taproot = TapRoot::key_and_script_path_single(agg_inner_key, sweep_path);

    // 7 Return the taproot
    Some(taproot)
}

/// Returns a scriptpubkey for the LiftV2 struct.
pub fn return_liftv2_scriptpubkey(account_key: [u8; 32], engine_key: [u8; 32]) -> Option<Bytes> {
    // 1 Construct the taproot
    let taproot = return_liftv2_taproot(account_key, engine_key)?;

    // 2 Return the scriptpubkey
    taproot.spk()
}

impl P2TR for LiftV2 {
    fn taproot(&self) -> Option<TapRoot> {
        return_liftv2_taproot(self.account_key, self.engine_key)
    }
}

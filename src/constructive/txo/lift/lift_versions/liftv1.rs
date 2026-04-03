use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::constructive::txn::ext::OutpointExt;
use bitcoin::{OutPoint, TxOut};
use hex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// A type alias for bytes.
type Bytes = Vec<u8>;

/// Non-interactive, but trusted, placeholder Lift implementation.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiftV1 {
    pub account_key: [u8; 32],
    pub engine_key: [u8; 32],
    pub outpoint: OutPoint,
    pub txout: TxOut,
}

impl LiftV1 {
    /// Creates a new LiftV1 struct.
    pub fn new(
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    ) -> LiftV1 {
        LiftV1 {
            account_key,
            engine_key,
            outpoint,
            txout,
        }
    }

    /// Validates the LiftV1 struct.
    ///
    /// Used by the `Engine` to validate the `LiftV1` is indeed a valid structure.
    pub fn validate(&self, account_key: [u8; 32], engine_key: [u8; 32]) -> bool {
        // 1 Validate the keys.
        if self.account_key != account_key || self.engine_key != engine_key {
            return false;
        }

        // 2 Validate the scriptpubkey.
        {
            // 2.1 Calculate the scriptpubkey.
            let calculated_scriptpubkey = match return_liftv1_scriptpubkey(account_key, engine_key)
            {
                Some(scriptpubkey) => scriptpubkey,
                None => return false,
            };

            // 2.2 Get the self scriptpubkey.
            let self_scriptpubkey = self.txout.script_pubkey.as_bytes();

            // 2.3 Validate the scriptpubkeys.
            if &calculated_scriptpubkey != self_scriptpubkey {
                return false;
            }
        }

        // 3 Return true.
        true
    }

    /// Returns a JSON representation of the Lift struct
    pub fn json(&self) -> Value {
        // 1 Construct the lift JSON object
        let mut obj = Map::new();

        // 2 Add lift version tag
        obj.insert(
            "version".to_string(),
            Value::String("v1_non_interactive".to_string()),
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

/// Returns a taproot for the LiftV1 struct.
pub fn return_liftv1_taproot(account_key: [u8; 32], engine_key: [u8; 32]) -> Option<TapRoot> {
    // 1 Construct the tapscript
    let mut tapscript = Vec::<u8>::new();

    // 2 Inscribe the account key for reference-only: <Account Key> OP_DROP
    {
        tapscript.push(0x20); // OP_PUSHDATA_32
        tapscript.extend(account_key); // Account Key 32-bytes
        tapscript.push(0x75); // OP_DROP
    }

    // 3 Only Engine can spend: <Engine Key> OP_CHECKSIG
    {
        tapscript.push(0x20); // OP_PUSHDATA_32
        tapscript.extend(engine_key); // Account Key 32-bytes
        tapscript.push(0xac); // OP_CHECKSIG
    }

    // 4 Construct the tapleaf
    let tapleaf = TapLeaf::new(tapscript);

    // 5 Construct the taproot
    let taproot = TapRoot::script_path_only_single(tapleaf);

    // 6 Return the taproot
    Some(taproot)
}

/// Returns a scriptpubkey for the LiftV1 struct.
pub fn return_liftv1_scriptpubkey(account_key: [u8; 32], engine_key: [u8; 32]) -> Option<Bytes> {
    // 1 Construct the taproot
    let taproot = return_liftv1_taproot(account_key, engine_key)?;

    // 2 Return the scriptpubkey
    taproot.spk()
}

impl P2TR for LiftV1 {
    fn taproot(&self) -> Option<TapRoot> {
        return_liftv1_taproot(self.account_key, self.engine_key)
    }
}

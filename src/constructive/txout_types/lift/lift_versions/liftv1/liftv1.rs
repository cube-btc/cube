use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::constructive::txn::ext::OutpointExt;
use crate::operative::run_args::chain::Chain;
use crate::transmutative::codec::address::encode_p2tr;
use bitcoin::{OutPoint, TxOut};
use hex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// A type alias for bytes.
type Bytes = Vec<u8>;

type TapLeafHash = [u8; 32];
type TapScript = Vec<u8>;
type ControlBlock = Vec<u8>;

/// Non-interactive, but trusted, placeholder Lift implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Validates the LiftV1 scriptpubkey.
    pub fn validate_scriptpubkey(&self) -> bool {
        // 1 Calculate the scriptpubkey.
        let calculated_scriptpubkey =
            match return_liftv1_scriptpubkey(self.account_key, self.engine_key) {
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

    /// Returns (tapleaf_hash, tapscript, control_block_bytes) for this LiftV1's P2TR script path.
    pub fn p2tr_script_path_spend_elements(&self) -> (TapLeafHash, TapScript, ControlBlock) {
        // For LiftV1 we construct a single-leaf, script-path-only TapRoot.
        let taproot = self
            .taproot()
            .expect("LiftV1 must always have a TapRoot for P2TR script-path spends");

        let tree = taproot
            .tree()
            .expect("TapRoot for LiftV1 must contain a TapTree");

        let leaves = tree.leaves();
        let tapleaf = leaves
            .first()
            .expect("TapTree for LiftV1 must contain at least one TapLeaf");

        let tapleaf_hash: TapLeafHash = tapleaf.tapleaf_hash();
        let tapscript: TapScript = tapleaf.tap_script();

        let control_block_bytes: ControlBlock = taproot
            .control_block(0)
            .expect("TapRoot for LiftV1 must produce a control block for leaf index 0")
            .to_vec();

        (tapleaf_hash, tapscript, control_block_bytes)
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

/// Returns a Bech32-encoded P2TR address for the LiftV1 struct.
pub fn return_liftv1_address(
    chain: Chain,
    account_key: [u8; 32],
    engine_key: [u8; 32],
) -> Option<String> {
    let taproot = return_liftv1_taproot(account_key, engine_key)?;
    let address = match encode_p2tr(chain, taproot.tweaked_key().unwrap().serialize_xonly()) {
        Some(address) => address,
        None => {
            return None;
        }
    };
    Some(address)
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

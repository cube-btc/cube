use crate::constructive::txn::ext::OutpointExt;
use crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1;
use crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2;
use bitcoin::{OutPoint, TxOut};
use hex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The Lift Bitcoin Transaction Output (TXO) type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lift {
    // Non-interactive, but trusted, placeholder Lift implementation.
    LiftV1(LiftV1),

    // MuSig2-based and thus interactive, but trustless Lift implementation.
    LiftV2(LiftV2),

    /// Unknown lift version, reserved for potential future use.
    Unknown {
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    },
}

impl Lift {
    /// Creates a new LiftV1 struct.
    pub fn new_liftv1(
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    ) -> Lift {
        Lift::LiftV1(LiftV1::new(account_key, engine_key, outpoint, txout))
    }

    /// Creates a new LiftV2 struct.
    pub fn new_liftv2(
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    ) -> Lift {
        Lift::LiftV2(LiftV2::new(account_key, engine_key, outpoint, txout))
    }

    /// Creates a new unknown-version lift (same on-chain fields, no v1/v2 scriptpubkey assumption).
    pub fn new_unknown(
        account_key: [u8; 32],
        engine_key: [u8; 32],
        outpoint: OutPoint,
        txout: TxOut,
    ) -> Lift {
        Lift::Unknown {
            account_key,
            engine_key,
            outpoint,
            txout,
        }
    }

    /// Returns the lift version.
    pub fn lift_version(&self) -> u8 {
        match self {
            Lift::Unknown { .. } => 0,
            Lift::LiftV1(_) => 1,
            Lift::LiftV2(_) => 2,
        }
    }

    /// Returns the account key.
    pub fn account_key(&self) -> [u8; 32] {
        match self {
            Lift::LiftV1(liftv1) => liftv1.account_key,
            Lift::LiftV2(liftv2) => liftv2.account_key,
            Lift::Unknown { account_key, .. } => *account_key,
        }
    }

    /// Returns the engine key.
    pub fn engine_key(&self) -> [u8; 32] {
        match self {
            Lift::LiftV1(liftv1) => liftv1.engine_key,
            Lift::LiftV2(liftv2) => liftv2.engine_key,
            Lift::Unknown { engine_key, .. } => *engine_key,
        }
    }

    /// Returns the outpoint.
    pub fn outpoint(&self) -> OutPoint {
        match self {
            Lift::LiftV1(liftv1) => liftv1.outpoint,
            Lift::LiftV2(liftv2) => liftv2.outpoint,
            Lift::Unknown { outpoint, .. } => *outpoint,
        }
    }

    /// Returns the txout.
    pub fn txout(&self) -> TxOut {
        match self {
            Lift::LiftV1(liftv1) => liftv1.txout.clone(),
            Lift::LiftV2(liftv2) => liftv2.txout.clone(),
            Lift::Unknown { txout, .. } => txout.clone(),
        }
    }

    /// Returns the lift value in satoshis.
    pub fn lift_value_in_satoshis(&self) -> u64 {
        self.txout().value.to_sat()
    }

    /// Returns a JSON representation of the Lift struct
    pub fn json(&self) -> Value {
        match self {
            Lift::LiftV1(liftv1) => liftv1.json(),
            Lift::LiftV2(liftv2) => liftv2.json(),
            Lift::Unknown {
                account_key,
                engine_key,
                outpoint,
                txout,
            } => {
                let mut obj = Map::new();
                obj.insert("version".to_string(), Value::String("unknown".to_string()));
                obj.insert(
                    "account_key".to_string(),
                    Value::String(hex::encode(account_key)),
                );
                obj.insert(
                    "engine_key".to_string(),
                    Value::String(hex::encode(engine_key)),
                );
                let outpoint_obj = {
                    let mut o = Map::new();
                    o.insert(
                        "txid".to_string(),
                        Value::String(hex::encode(outpoint.txhash())),
                    );
                    o.insert("vout".to_string(), Value::Number(outpoint.vout.into()));
                    o
                };
                obj.insert("outpoint".to_string(), Value::Object(outpoint_obj));
                let txout_obj = {
                    let mut o = Map::new();
                    o.insert(
                        "satoshis".to_string(),
                        Value::Number(txout.value.to_sat().into()),
                    );
                    o.insert(
                        "scriptpubkey".to_string(),
                        Value::String(hex::encode(txout.script_pubkey.as_bytes())),
                    );
                    o
                };
                obj.insert("txout".to_string(), Value::Object(txout_obj));
                Value::Object(obj)
            }
        }
    }
}

use crate::constructive::txo::lift::lift_versions::liftv1::LiftV1;
use crate::constructive::txo::lift::lift_versions::liftv2::LiftV2;
use bitcoin::{OutPoint, TxOut};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The Lift enum.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lift {
    // Non-interactive, but trusted, placeholder Lift implementation.
    LiftV1(LiftV1),

    // MuSig2-based and thus interactive, but trustless Lift implementation.
    LiftV2(LiftV2),
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

    /// Returns the lift version.
    pub fn lift_version(&self) -> u8 {
        match self {
            Lift::LiftV1(_) => 1,
            Lift::LiftV2(_) => 2,
        }
    }

    /// Returns the account key.
    pub fn account_key(&self) -> [u8; 32] {
        match self {
            Lift::LiftV1(liftv1) => liftv1.account_key,
            Lift::LiftV2(liftv2) => liftv2.account_key,
        }
    }

    /// Returns the engine key.
    pub fn engine_key(&self) -> [u8; 32] {
        match self {
            Lift::LiftV1(liftv1) => liftv1.engine_key,
            Lift::LiftV2(liftv2) => liftv2.engine_key,
        }
    }

    /// Returns the outpoint.
    pub fn outpoint(&self) -> OutPoint {
        match self {
            Lift::LiftV1(liftv1) => liftv1.outpoint,
            Lift::LiftV2(liftv2) => liftv2.outpoint,
        }
    }

    /// Returns the txout.
    pub fn txout(&self) -> TxOut {
        match self {
            Lift::LiftV1(liftv1) => liftv1.txout.clone(),
            Lift::LiftV2(liftv2) => liftv2.txout.clone(),
        }
    }

    /// Returns a JSON representation of the Lift struct
    pub fn json(&self) -> Value {
        match self {
            Lift::LiftV1(liftv1) => liftv1.json(),
            Lift::LiftV2(liftv2) => liftv2.json(),
        }
    }
}

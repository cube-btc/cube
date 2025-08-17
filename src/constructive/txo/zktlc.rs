use crate::constructive::taproot::{TapLeaf, TapRoot, P2TR};
use crate::transmutative::codec::csv::{CSVEncode, CSVFlag};
use crate::transmutative::musig::keyagg::MusigKeyAggCtx;
use crate::transmutative::secp::into::IntoScalar;
use bitcoin::OutPoint;
use secp::Point;
use serde::{Deserialize, Serialize};

type Bytes = Vec<u8>;

const RECHARGE_HEIGHT_OFFSET: u32 = 144 * 45;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZKTLC {
    account: Point,
    engine: Point,
    outpoint: Option<OutPoint>,
    value: Option<u64>,
    at_rollup_height: Option<u32>,
    at_bitcoin_height: Option<u32>,
}

impl ZKTLC {
    /// Creates a new VTXO.     
    pub fn new(
        account: Point,
        engine: Point,
        outpoint: Option<OutPoint>,
        value: Option<u64>,
        at_rollup_height: Option<u32>,
        at_bitcoin_height: Option<u32>,
    ) -> ZKTLC {
        ZKTLC {
            account,
            engine,
            outpoint,
            value,
            at_rollup_height,
            at_bitcoin_height,
        }
    }

    /// Serializes the VTXO into a vector of bytes.
    pub fn serialize(&self) -> Vec<u8> {
        match serde_json::to_vec(self) {
            Ok(bytes) => bytes,
            Err(_) => vec![],
        }
    }

    /// Returns the account key of the VTXO.
    pub fn account_key(&self) -> Point {
        self.account.clone()
    }

    /// Returns the engine key of the VTXO.
    pub fn engine_key(&self) -> Point {
        self.engine
    }

    /// Returns the outpoint of the VTXO.
    pub fn outpoint(&self) -> Option<OutPoint> {
        self.outpoint
    }

    /// Returns the sats value of the VTXO.
    pub fn value(&self) -> Option<u64> {
        self.value
    }

    /// Returns the rollup block height which the VTXO was created.
    pub fn at_rollup_height(&self) -> Option<u32> {
        self.at_rollup_height
    }

    /// Returns the Bitcoin block height which the VTXO was confirmed.
    pub fn at_bitcoin_height(&self) -> Option<u32> {
        self.at_bitcoin_height
    }

    /// Returns the keys of the VTXO.
    pub fn keys(&self) -> Vec<Point> {
        let mut keys = Vec::<Point>::new();

        keys.push(self.account_key());
        keys.push(self.engine_key());

        keys
    }

    /// Returns the aggregated inner key of the VTXO.
    pub fn agg_inner_key(&self) -> Option<Point> {
        let keys = self.keys();
        let key_agg_ctx = MusigKeyAggCtx::new(&keys, None)?;
        let agg_inner_key = key_agg_ctx.agg_inner_key();

        Some(agg_inner_key)
    }

    /// Returns the key aggregation context of the VTXO.
    pub fn key_agg_ctx(&self) -> Option<MusigKeyAggCtx> {
        let taproot = self.taproot()?;
        let tweak = taproot.tap_tweak().into_scalar().ok()?;
        let keys = self.keys();
        let key_agg_ctx = MusigKeyAggCtx::new(&keys, Some(tweak))?;

        Some(key_agg_ctx)
    }

    /// Returns true if the VTXO is rechargeable.
    pub fn is_rechargeable(&self, current_bitcoin_height: u32) -> Option<bool> {
        // Calculate the height at which the recharge allowance begins.
        let recharge_allowance_height = current_bitcoin_height - RECHARGE_HEIGHT_OFFSET;

        // Get the Bitcoin block height at which the VTXO was confirmed.
        let at_bitcoin_height = self.at_bitcoin_height()?;

        // Check if the VTXO is rechargeable.
        let is_rechargeable = at_bitcoin_height <= recharge_allowance_height;

        // Return the result.
        Some(is_rechargeable)
    }
}

// TODO: re-factor this for ZKTLC.
impl P2TR for ZKTLC {
    fn taproot(&self) -> Option<TapRoot> {
        //// Inner Key: (Self + Operator)
        let agg_inner_key = self.agg_inner_key()?;

        //// Sweep Path: (Operator after 3 months)
        let mut sweep_path_script = Vec::<u8>::new();
        sweep_path_script.extend(Bytes::csv_script(CSVFlag::CSVThreeMonths)); // Relative Timelock
        sweep_path_script.push(0x20); // OP_PUSHDATA_32
        sweep_path_script.extend(self.account_key().serialize_xonly()); // Account Key 32-bytes
        sweep_path_script.push(0xac); // OP_CHECKSIG

        let sweep_path = TapLeaf::new(sweep_path_script);

        Some(TapRoot::key_and_script_path_single(
            agg_inner_key,
            sweep_path,
        ))
    }

    fn spk(&self) -> Option<Bytes> {
        self.taproot()?.spk()
    }
}

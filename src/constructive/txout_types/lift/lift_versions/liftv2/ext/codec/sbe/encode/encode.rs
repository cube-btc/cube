use crate::constructive::txn::ext::OutpointExt;
use crate::constructive::txn::ext::TxOutExt;

use crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2;

/// Leading SBE byte for [`LiftV2`] (`0x01`).
const LIFT_V2_SBE_VARIANT_DISCRIMINANT: u8 = 0x01;

impl LiftV2 {
    /// Encodes this `LiftV2` as Structural Byte-scope Encoding (SBE) bytes.
    ///
    /// Layout: `0x01` (variant) + 32-byte account key + 32-byte engine key + 36-byte outpoint
    /// (`OutpointExt::bytes_36`) + `TxOut` bytes (`TxOutExt::bytes()`).
    pub fn encode_sbe(&self) -> Vec<u8> {
        // 1 Allocate output with capacity for the discriminant, fixed fields, and variable `TxOut` tail.
        let mut out = Vec::with_capacity(1 + 32 + 32 + 36 + self.txout.bytes().len());

        // 2 Push the `LiftV2` variant discriminant.
        out.push(LIFT_V2_SBE_VARIANT_DISCRIMINANT);

        // 3 Encode the Schnorr account key and engine key (32 bytes each).
        out.extend_from_slice(&self.account_key);
        out.extend_from_slice(&self.engine_key);

        // 4 Encode the outpoint as 36 bytes (`OutpointExt::bytes_36`).
        out.extend_from_slice(&self.outpoint.bytes_36());

        // 5 Encode the `TxOut` via `TxOutExt::bytes` (8-byte value LE, 1-byte script length, script).
        out.extend_from_slice(&self.txout.bytes());

        // 6 Return the buffer.
        out
    }
}

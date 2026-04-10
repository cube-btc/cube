use crate::constructive::txn::ext::OutpointExt;
use crate::constructive::txn::ext::TxOutExt;

use crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1;

impl LiftV1 {
    /// Encodes this `LiftV1` as Structural Byte-scope Encoding (SBE) bytes.
    pub fn encode_sbe(&self) -> Vec<u8> {
        // 1 Allocate output with capacity for the discriminant, fixed fields, and variable `TxOut` tail.
        let mut out = Vec::with_capacity(1 + 32 + 32 + 36 + self.txout.bytes().len());

        // 2 Push the `LiftV1` variant discriminant.
        out.push(0x01);

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

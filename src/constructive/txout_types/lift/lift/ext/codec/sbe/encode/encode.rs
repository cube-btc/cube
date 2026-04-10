use crate::constructive::txn::ext::{OutpointExt, TxOutExt};
use crate::constructive::txo::lift::lift::Lift;

impl Lift {
    /// Structural Byte-scope Encoding (SBE) for `Lift`.
    ///
    /// This function encodes a `Lift` by delegating to the inner variant encoder or writing the shared body layout.
    ///
    /// The first byte is `0x00` for [`Lift::Unknown`](Lift::Unknown),
    /// `0x01` for [`LiftV1`](crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1),
    /// or `0x02` for [`LiftV2`](crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2)
    /// (same tail layout for all three after the discriminant).
    pub fn encode_sbe(&self) -> Vec<u8> {
        // 1 Encode by variant.
        match self {
            // 1.a `LiftV1` payload (leading `0x01` plus shared body).
            Lift::LiftV1(liftv1) => liftv1.encode_sbe(),
            // 1.b `LiftV2` payload (leading `0x02` plus shared body).
            Lift::LiftV2(liftv2) => liftv2.encode_sbe(),
            // 1.c `Unknown`: `0x00` plus shared body (account key, engine key, outpoint, `TxOut` bytes).
            Lift::Unknown {
                account_key,
                engine_key,
                outpoint,
                txout,
            } => {
                let mut out = Vec::with_capacity(1 + 32 + 32 + 36 + txout.bytes().len());
                out.push(0x00);
                out.extend_from_slice(account_key);
                out.extend_from_slice(engine_key);
                out.extend_from_slice(&outpoint.bytes_36());
                out.extend_from_slice(&txout.bytes());
                out
            }
        }
    }
}

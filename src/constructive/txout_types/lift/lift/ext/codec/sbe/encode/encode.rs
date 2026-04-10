use crate::constructive::txo::lift::lift::Lift;

impl Lift {
    /// Structural Byte-scope Encoding (SBE) for `Lift`.
    ///
    /// This function encodes a `Lift` by delegating to the inner variant encoder.
    ///
    /// The first byte is `0x00` for [`LiftV1`](crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1)
    /// or `0x01` for [`LiftV2`](crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2);
    /// the remainder is encoded by the corresponding variant's [`LiftV1::encode_sbe`] / [`LiftV2::encode_sbe`].
    pub fn encode_sbe(&self) -> Vec<u8> {
        // 1 Delegate to the variant-specific SBE encoder (`LiftV1` / `LiftV2`).
        match self {
            // 1.a `LiftV1` payload (leading `0x00` plus shared body).
            Lift::LiftV1(liftv1) => liftv1.encode_sbe(),
            // 1.b `LiftV2` payload (leading `0x01` plus shared body).
            Lift::LiftV2(liftv2) => liftv2.encode_sbe(),
        }
    }
}

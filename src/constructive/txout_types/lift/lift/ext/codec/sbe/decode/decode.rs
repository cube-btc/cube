use crate::constructive::txo::lift::lift::ext::codec::sbe::decode::error::decode_error::LiftSBEDecodeError;
use crate::constructive::txo::lift::lift::Lift;
use crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1;
use crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2;

impl Lift {
    /// Decodes a `Lift` from Structural Byte-scope Encoding (SBE) bytes produced by [`Lift::encode_sbe`].
    ///
    /// The first byte selects `LiftV1` or `LiftV2`; the remainder is parsed by the corresponding variant decoder.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Lift, LiftSBEDecodeError> {
        // 1 Ensure there is a leading variant discriminant byte.
        if bytes.is_empty() {
            return Err(LiftSBEDecodeError::LiftSBEVariantDiscriminantMissingError);
        }

        // 2 Branch on the discriminant and delegate to `LiftV1` / `LiftV2` decoders.
        match bytes[0] {
            // 2.a `LiftV1` (`0x00`).
            0x00 => LiftV1::decode_sbe(bytes)
                .map(Lift::LiftV1)
                .map_err(LiftSBEDecodeError::LiftV1),
            // 2.b `LiftV2` (`0x01`).
            0x01 => LiftV2::decode_sbe(bytes)
                .map(Lift::LiftV2)
                .map_err(LiftSBEDecodeError::LiftV2),
            // 2.c Unknown discriminant.
            b => Err(LiftSBEDecodeError::UnknownLiftSBEVariantDiscriminantByteError(b)),
        }
    }
}

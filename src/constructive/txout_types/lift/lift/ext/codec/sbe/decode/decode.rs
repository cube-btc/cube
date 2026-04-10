use crate::constructive::txo::lift::lift::ext::codec::sbe::decode::error::decode_error::LiftSBEDecodeError;
use crate::constructive::txo::lift::lift::Lift;
use crate::constructive::txo::lift::lift_versions::liftv1::ext::codec::sbe::decode::decode::decode_lift_v1_sbe_body;
use crate::constructive::txo::lift::lift_versions::liftv1::liftv1::LiftV1;
use crate::constructive::txo::lift::lift_versions::liftv2::liftv2::LiftV2;

impl Lift {
    /// Decodes a `Lift` from Structural Byte-scope Encoding (SBE) bytes produced by [`Lift::encode_sbe`].
    ///
    /// The first byte selects `LiftV1`, `LiftV2`, or `Unknown`; the remainder is parsed accordingly.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Lift, LiftSBEDecodeError> {
        // 1 Ensure there is a leading variant discriminant byte.
        if bytes.is_empty() {
            return Err(LiftSBEDecodeError::LiftSBEVariantDiscriminantMissingError);
        }

        // 2 Branch on the discriminant.
        match bytes[0] {
            // 2.a `Unknown` (`0x00`): shared body only (no v1/v2 script template).
            0x00 => {
                let payload = &bytes[1..];
                let (account_key, engine_key, outpoint, txout) = decode_lift_v1_sbe_body(payload)
                    .map_err(LiftSBEDecodeError::LiftUnknown)?;
                Ok(Lift::Unknown {
                    account_key,
                    engine_key,
                    outpoint,
                    txout,
                })
            }
            // 2.b `LiftV1` (`0x01`).
            0x01 => LiftV1::decode_sbe(bytes)
                .map(Lift::LiftV1)
                .map_err(LiftSBEDecodeError::LiftV1),
            // 2.c `LiftV2` (`0x02`).
            0x02 => LiftV2::decode_sbe(bytes)
                .map(Lift::LiftV2)
                .map_err(LiftSBEDecodeError::LiftV2),
            // 2.d Unknown discriminant.
            b => Err(LiftSBEDecodeError::UnknownLiftSBEVariantDiscriminantByteError(b)),
        }
    }
}

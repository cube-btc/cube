use crate::constructive::txo::lift::lift_versions::liftv1::ext::codec::sbe::decode::error::LiftV1SBEDecodeError;
use crate::constructive::txo::lift::lift_versions::liftv2::ext::codec::sbe::decode::error::LiftV2SBEDecodeError;

/// Errors that can occur when decoding a [`Lift`](crate::constructive::txo::lift::lift::Lift) from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiftSBEDecodeError {
    // Top-level `Lift` discriminant.
    /// The buffer was empty, so the leading `Lift` SBE variant byte could not be read.
    LiftSBEVariantDiscriminantMissingError,
    /// The leading byte is not a known `Lift` SBE variant discriminant (`0x00` for `LiftV1`, `0x01` for `LiftV2`).
    UnknownLiftSBEVariantDiscriminantByteError(u8),

    // Variant-specific decode failures.
    /// Decoding failed for an SBE buffer tagged as `LiftV1`.
    LiftV1(LiftV1SBEDecodeError),
    /// Decoding failed for an SBE buffer tagged as `LiftV2`.
    LiftV2(LiftV2SBEDecodeError),
}

use crate::constructive::entry::entry_types::liftup::ext::codec::sbe::decode::error::LiftupSBEDecodeError;

/// Errors that can occur when decoding an `Entry` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntrySBEDecodeError {
    // Top-level `Entry` discriminant.
    /// The buffer was empty, so the leading `Entry` SBE variant byte could not be read.
    EntrySBEVariantDiscriminantMissingError,
    /// The leading byte is not `0x01` (`Call`) or `0x04` (`Liftup`).
    UnknownEntrySBEVariantDiscriminantByteError(u8),

    // `Liftup` payload after `0x04`.
    /// Decoding the `Liftup` SBE payload failed.
    LiftupSBEDecodeError(LiftupSBEDecodeError),
}

use crate::constructive::entry::entry_kinds::liftup::ext::codec::sbe::decode::error::LiftupSBEDecodeError;
use crate::constructive::entry::entry_kinds::r#move::ext::codec::sbe::decode::error::decode_error::MoveSBEDecodeError;
use crate::constructive::entry::entry_kinds::swapout::ext::codec::sbe::decode::error::decode_error::SwapoutSBEDecodeError;

/// Errors that can occur when decoding an `Entry` from Structural Byte-scope Encoding (SBE) bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntrySBEDecodeError {
    // Top-level `Entry` discriminant.
    /// The buffer was empty, so the leading `Entry` SBE variant byte could not be read.
    EntrySBEVariantDiscriminantMissingError,
    /// The leading byte is not `0x00` (`Move`), `0x01` (`Call`) or `0x04` (`Liftup`).
    UnknownEntryKindByteError(u8),

    // `Move` SBE (full buffer, leading `0x00` is part of `Move` SBE).
    /// Decoding the `Move` SBE bytes failed.
    MoveSBEDecodeError(MoveSBEDecodeError),

    // `Liftup` SBE (full buffer, leading `0x04` is part of `Liftup` SBE).
    /// Decoding the `Liftup` SBE bytes failed.
    LiftupSBEDecodeError(LiftupSBEDecodeError),
    /// Decoding the `Swapout` SBE bytes failed.
    SwapoutSBEDecodeError(SwapoutSBEDecodeError),
}

use crate::constructive::entry::entry::entry::Entry;

use super::error::EntrySBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Entry {
    /// Structural Byte-scope Encoding (SBE) for `Entry`.
    pub fn encode_sbe(&self) -> Result<Bytes, EntrySBEEncodeError> {
        // 1 Match on the `Entry` variant.
        match self {
            // 1.a The `Entry` is a `Call` — SBE not implemented.
            Entry::Call(_) => panic!(
                "Entry::encode_sbe: Call SBE is not implemented (discriminant 0x01 reserved)"
            ),

            // 1.b The `Entry` is a `Liftup` — SBE is the `Liftup` encoding (leading `0x04` is inside `Liftup::encode_sbe`).
            Entry::Liftup(liftup) => liftup
                .encode_sbe()
                .map_err(|err| EntrySBEEncodeError::LiftupSBEEncodeError(err)),
        }
    }
}

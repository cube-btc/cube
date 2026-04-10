use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry_types::liftup::liftup::Liftup;

use super::error::EntrySBEDecodeError;

impl Entry {
    /// Decodes an `Entry` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Self, EntrySBEDecodeError> {
        // 1 Ensure there is at least one byte for the `Entry` variant discriminant.
        if bytes.is_empty() {
            return Err(EntrySBEDecodeError::EntrySBEVariantDiscriminantMissingError);
        }

        // 2 Read the discriminant and slice the payload after it.
        let variant_tag = bytes[0];
        let payload = &bytes[1..];

        // 3 Match on the discriminant.
        match variant_tag {
            // 3.a `Call` (`0x01`) — SBE not implemented.
            0x01 => {
                panic!(
                    "Entry::decode_sbe: Call SBE is not implemented (discriminant 0x01 reserved)"
                );
            }

            // 3.b `Liftup` (`0x04`).
            0x04 => {
                // 3.b.1 Decode the `Liftup` from the remaining bytes.
                let liftup = Liftup::decode_sbe(payload)
                    .map_err(|err| EntrySBEDecodeError::LiftupSBEDecodeError(err))?;

                // 3.b.2 Return the `Entry`.
                Ok(Entry::Liftup(liftup))
            }

            // 3.c Unknown discriminant.
            b => Err(EntrySBEDecodeError::UnknownEntrySBEVariantDiscriminantByteError(b)),
        }
    }
}

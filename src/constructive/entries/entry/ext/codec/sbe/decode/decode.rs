use crate::constructive::entry::entry::entry::Entry;
use crate::constructive::entry::entry_kinds::liftup::liftup::Liftup;
use crate::constructive::entry::entry_kinds::r#move::r#move::Move;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;

use super::error::EntrySBEDecodeError;

impl Entry {
    /// Decodes an `Entry` from Structural Byte-scope Encoding (SBE) bytes.
    pub fn decode_sbe(bytes: &[u8]) -> Result<Self, EntrySBEDecodeError> {
        // 1 Ensure there is at least one byte for the `Entry` variant discriminant.
        if bytes.is_empty() {
            return Err(EntrySBEDecodeError::EntrySBEVariantDiscriminantMissingError);
        }

        // 2 Read the entry kind byte.
        let entry_kind_byte = bytes[0];

        // 3 Match on the entry type byte.
        match entry_kind_byte {
            // 3.a `Move` (`0x00`): decode from the full buffer (`Move::decode_sbe` consumes the tag).
            0x00 => {
                let move_entry = Move::decode_sbe(bytes)
                    .map_err(EntrySBEDecodeError::MoveSBEDecodeError)?;

                Ok(Entry::Move(move_entry))
            }

            // 3.b `Call` (`0x01`) — SBE not implemented.
            0x01 => {
                panic!(
                    "Entry::decode_sbe: Call SBE is not implemented (discriminant 0x01 reserved)"
                );
            }

            // 3.c `Liftup` (`0x04`): decode from the full buffer (`Liftup::decode_sbe` consumes the tag).
            0x04 => {
                let liftup = Liftup::decode_sbe(bytes)
                    .map_err(|err| EntrySBEDecodeError::LiftupSBEDecodeError(err))?;

                Ok(Entry::Liftup(liftup))
            }
            0x05 => {
                let swapout = Swapout::decode_sbe(bytes)
                    .map_err(EntrySBEDecodeError::SwapoutSBEDecodeError)?;
                Ok(Entry::Swapout(swapout))
            }

            // 3.d Unknown entry kind byte.
            b => Err(EntrySBEDecodeError::UnknownEntryKindByteError(b)),
        }
    }
}

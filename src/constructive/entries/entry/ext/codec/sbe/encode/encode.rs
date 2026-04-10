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

            // 1.b The `Entry` is a `Liftup`.
            Entry::Liftup(liftup) => {
                // 1.b.1 Encode the inner `Liftup` SBE payload.
                let liftup_bytes = liftup
                    .encode_sbe()
                    .map_err(|err| EntrySBEEncodeError::LiftupSBEEncodeError(err))?;

                // 1.b.2 Allocate the output: discriminant plus `Liftup` payload.
                let mut out = Bytes::with_capacity(1 + liftup_bytes.len());

                // 1.b.3 Push the `Liftup` entry discriminant.
                out.push(0x04);

                // 1.b.4 Append the `Liftup` SBE bytes.
                out.extend_from_slice(&liftup_bytes);

                // 1.b.5 Return the buffer.
                Ok(out)
            }
        }
    }
}

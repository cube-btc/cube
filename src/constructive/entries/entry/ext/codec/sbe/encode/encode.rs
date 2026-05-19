use crate::constructive::entry::entry::entry::Entry;

use super::error::EntrySBEEncodeError;

/// A type alias for a vector of bytes.
type Bytes = Vec<u8>;

impl Entry {
    /// Structural Byte-scope Encoding (SBE) for `Entry`.
    pub fn encode_sbe(&self) -> Result<Bytes, EntrySBEEncodeError> {
        // 1 Match on the `Entry` variant.
        match self {
            // 1.0 The `Entry` is a `Move`.
            Entry::Move(move_entry) => move_entry
                .encode_sbe()
                .map_err(EntrySBEEncodeError::MoveSBEEncodeError),

            Entry::Call(call) => call
                .encode_sbe()
                .map_err(EntrySBEEncodeError::CallSBEEncodeError),

            // 1.b The `Entry` is a `Liftup` — SBE is the `Liftup` encoding (leading `0x04` is inside `Liftup::encode_sbe`).
            Entry::Liftup(liftup) => liftup
                .encode_sbe()
                .map_err(|err| EntrySBEEncodeError::LiftupSBEEncodeError(err)),
            Entry::Swapout(swapout) => swapout
                .encode_sbe()
                .map_err(EntrySBEEncodeError::SwapoutSBEEncodeError),
            Entry::Deploy(deploy) => deploy
                .encode_sbe()
                .map_err(EntrySBEEncodeError::DeploySBEEncodeError),
            Entry::Config(config) => config
                .encode_sbe()
                .map_err(EntrySBEEncodeError::ConfigSBEEncodeError),
        }
    }
}

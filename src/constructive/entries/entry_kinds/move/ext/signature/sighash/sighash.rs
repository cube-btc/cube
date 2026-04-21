use crate::constructive::entries::entry_kinds::r#move::ext::signature::sighash::error::sighash_error::MoveSighashError;
use crate::constructive::entries::entry_kinds::r#move::r#move::Move;
use crate::transmutative::hash::{Hash, HashTag};

impl Move {
    /// Returns the signature message (sighash) for the `Move`.
    pub fn sighash(&self) -> Result<[u8; 32], MoveSighashError> {
        // 1 Encode the `Move` as SBE bytes for the sighash preimage.
        let sighash_preimage = self
            .encode_sbe()
            .map_err(MoveSighashError::SBEEncodeError)?;

        // 2 Hash the sighash preimage with the `MoveEntrySighash` tag.
        let sighash = sighash_preimage.hash(Some(HashTag::MoveEntrySighash));

        // 3 Return the sighash.
        Ok(sighash)
    }
}

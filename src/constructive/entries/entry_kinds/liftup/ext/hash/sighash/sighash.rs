use crate::constructive::entries::entry_kinds::liftup::ext::hash::sighash::error::sighash_error::LiftupSighashError;
use crate::constructive::entries::entry_kinds::liftup::liftup::Liftup;
use crate::transmutative::hash::{Hash, HashTag};

impl Liftup {
    /// Returns the signature message (sighash) for the `Liftup`.
    pub fn sighash(&self) -> Result<[u8; 32], LiftupSighashError> {
        // 1 Encode the `Liftup` as SBE bytes for the sighash preimage.
        let sighash_preimage = self
            .encode_sbe()
            .map_err(|err| LiftupSighashError::SBEEncodeError(err))?;

        // 2 Hash the sighash preimage with the 'LiftupEntrySighash' tag.
        let sighash = sighash_preimage.hash(Some(HashTag::LiftupEntrySighash));

        // 3 Return the sighash.
        Ok(sighash)
    }
}

use crate::constructive::entries::entry_kinds::call::call::Call;
use crate::constructive::entries::entry_kinds::call::ext::signature::sighash::error::sighash_error::CallSighashError;
use crate::transmutative::hash::{Hash, HashTag};

impl Call {
    /// Returns the signature message (sighash) for the `Call`.
    pub fn sighash(&self) -> Result<[u8; 32], CallSighashError> {
        // 1 Encode the `Call` as SBE bytes for the sighash preimage.
        let sighash_preimage = self
            .encode_sbe()
            .map_err(CallSighashError::SBEEncodeError)?;

        // 2 Hash the sighash preimage with the `CallEntrySighash` tag.
        let sighash = sighash_preimage.hash(Some(HashTag::CallEntrySighash));

        // 3 Return the sighash.
        Ok(sighash)
    }
}

use crate::constructive::entry::entry_kinds::deploy::deploy::Deploy;
use crate::constructive::entry::entry_kinds::deploy::ext::signature::sighash::error::sighash_error::DeploySighashError;
use crate::transmutative::hash::{Hash, HashTag};

impl Deploy {
    /// Returns the signature message (sighash) for the `Deploy`.
    pub fn sighash(&self) -> Result<[u8; 32], DeploySighashError> {
        // Use program contract-id as sighash base, plus deploy fields.
        let mut preimage = Vec::<u8>::new();
        preimage.extend(self.program.contract_id());
        preimage.extend(self.root_account.account_key());
        preimage.extend(self.initial_balance.to_le_bytes());
        preimage.extend(self.target.targeted_at_batch_height.to_le_bytes());
        Ok(preimage.hash(Some(HashTag::DeployEntrySighash)))
    }
}

use crate::constructive::entry::entry_kinds::config::config::Config;
use crate::constructive::entry::entry_kinds::config::ext::signature::sighash::error::sighash_error::ConfigSighashError;
use crate::transmutative::hash::{Hash, HashTag};

impl Config {
    /// Returns the signature message (sighash) for the `Config`.
    pub fn sighash(&self) -> Result<[u8; 32], ConfigSighashError> {
        let sighash_preimage = self
            .encode_sbe()
            .map_err(ConfigSighashError::SBEEncodeError)?;
        Ok(sighash_preimage.hash(Some(HashTag::ConfigEntrySighash)))
    }
}

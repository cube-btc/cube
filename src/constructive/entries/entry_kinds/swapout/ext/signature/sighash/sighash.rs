use crate::constructive::entry::entry_kinds::swapout::ext::codec::sbe::encode::error::encode_error::SwapoutSBEEncodeError;
use crate::constructive::entry::entry_kinds::swapout::ext::signature::sighash::error::sighash_error::SwapoutSighashError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;
use crate::transmutative::hash::{Hash, HashTag};

impl Swapout {
    /// Returns the signature message (sighash) for the `Swapout`.
    ///
    /// Preimage bytes match [`Swapout::encode_sbe`] (duplicated here so signing does not depend on calling `encode_sbe`).
    pub fn sighash(&self) -> Result<[u8; 32], SwapoutSighashError> {
        let root_bytes = self.root_account.encode_sbe();
        let root_len_u32 = u32::try_from(root_bytes.len()).map_err(|_| {
            SwapoutSighashError::SBEEncodeError(
                SwapoutSBEEncodeError::SwapoutSBERootAccountPayloadTooLargeForU32LengthPrefix {
                    len: root_bytes.len(),
                },
            )
        })?;

        let mut preimage = Vec::new();
        preimage.push(0x05);
        preimage.extend_from_slice(&root_len_u32.to_le_bytes());
        preimage.extend_from_slice(&root_bytes);
        preimage.extend_from_slice(&self.amount.to_le_bytes());
        preimage.extend_from_slice(&self.target.encode_sbe());

        match &self.pinless_self {
            PinlessSelf::Default(_) => {
                preimage.push(0x00);
            }
            PinlessSelf::Unknown(pinless_self_unknown) => {
                preimage.push(0x01);
                preimage.extend_from_slice(&pinless_self_unknown.custom_scriptpubkey);
            }
        }

        Ok(preimage.hash(Some(HashTag::SwapoutEntrySighash)))
    }
}

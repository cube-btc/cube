use crate::constructive::entry::entry_kinds::swapout::ext::codec::sbe::encode::error::encode_error::SwapoutSBEEncodeError;
use crate::constructive::entry::entry_kinds::swapout::swapout::Swapout;
use crate::constructive::txout_types::pinless_self::PinlessSelf;

impl Swapout {
    /// Structural Byte-scope Encoding (SBE) encoding for `Swapout`.
    pub fn encode_sbe(&self) -> Result<Vec<u8>, SwapoutSBEEncodeError> {
        let root_bytes = self.root_account.encode_sbe();
        let root_len_u32 = u32::try_from(root_bytes.len()).map_err(|_| {
            SwapoutSBEEncodeError::SwapoutSBERootAccountPayloadTooLargeForU32LengthPrefix {
                len: root_bytes.len(),
            }
        })?;

        let mut bytes = Vec::new();
        bytes.push(0x05);
        bytes.extend_from_slice(&root_len_u32.to_le_bytes());
        bytes.extend_from_slice(&root_bytes);
        bytes.extend_from_slice(&self.target.encode_sbe());
        bytes.extend_from_slice(&self.amount.to_le_bytes());

        match &self.pinless_self {
            PinlessSelf::Default(_) => {
                bytes.push(0x00);
            }
            PinlessSelf::Unknown(pinless_self_unknown) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&pinless_self_unknown.custom_scriptpubkey);
            }
        }
        Ok(bytes)
    }
}

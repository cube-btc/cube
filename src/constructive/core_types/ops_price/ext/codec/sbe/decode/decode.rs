use crate::constructive::core_types::ops_price::ops_price::OpsPrice;

use super::error::decode_error::OpsPriceSBEDecodeError;

impl OpsPrice {
    /// Decodes an `OpsPrice` from Structural Byte-scope Encoding (SBE) bytes produced by [`OpsPrice::encode_sbe`].
    pub fn decode_sbe(bytes: &[u8]) -> Result<OpsPrice, OpsPriceSBEDecodeError> {
        let arr: [u8; 8] =
            bytes
                .try_into()
                .map_err(|_| OpsPriceSBEDecodeError::OpsPriceSBEInvalidPayloadLength {
                    got: bytes.len(),
                })?;

        Ok(OpsPrice::new(u64::from_le_bytes(arr)))
    }
}

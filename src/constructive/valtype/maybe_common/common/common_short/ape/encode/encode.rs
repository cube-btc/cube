use crate::constructive::valtype::maybe_common::common::common_short::common_short::{
    CommonShortVal, COMMON_SHORT_BITSIZE,
};
use crate::constructive::valtype::maybe_common::common::common_short::ape::encode::error::encode_error::CommonShortValAPEEncodeError;
use crate::constructive::valtype::u8_ext::U8Ext;
use bit_vec::BitVec;

impl CommonShortVal {
    /// Airly Payload Encoding (APE) encoding for `CommonShortVal`.
    ///
    /// This function encodes a `CommonShortVal` into an Airly Payload Encoding (APE) bit vector.
    /// The `CommonShortVal` can be a u32.
    ///
    /// # Arguments
    /// * `&self` - The `CommonShortVal` to encode.
    pub fn encode_ape(&self) -> Result<BitVec, CommonShortValAPEEncodeError> {
        // 1 Get the index.
        let index = self.index();

        // 2 Convert index to a bits with the bitsize of 6.
        let bits = u8::to_bits(index, COMMON_SHORT_BITSIZE)
            .ok_or(CommonShortValAPEEncodeError::U8ExtToBitsError)?;

        // 3 Return the bit vector.
        Ok(bits)
    }
}

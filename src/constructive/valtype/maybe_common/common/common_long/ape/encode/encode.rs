use crate::constructive::valtype::maybe_common::common::common_long::{
    ape::encode::error::encode_error::CommonLongValAPEEncodeError,
    common_long::{CommonLongVal, COMMON_LONG_BITSIZE},
};
use crate::constructive::valtype::u8_ext::U8Ext;
use bit_vec::BitVec;

impl CommonLongVal {
    /// Airly Payload Encoding (APE) encoding for `CommonLongVal`.
    ///
    /// This function encodes a `CommonLongVal` into an Airly Payload Encoding (APE) bit vector.
    /// The `CommonLongVal` can be a u64.
    ///
    /// # Arguments
    /// * `&self` - The `CommonLongVal` to encode.
    pub fn encode_ape(&self) -> Result<BitVec, CommonLongValAPEEncodeError> {
        // 1 Get the index.
        let index = self.index();

        // 2 Convert index to a bits with the bitsize of 7.
        let bits = u8::to_bits(index, COMMON_LONG_BITSIZE)
            .ok_or(CommonLongValAPEEncodeError::U8ExtToBitsError)?;

        // 3 Return the bit vector.
        Ok(bits)
    }
}

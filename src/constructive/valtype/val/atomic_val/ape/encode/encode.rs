use crate::constructive::valtype::u8_ext::U8Ext;
use crate::constructive::valtype::val::atomic_val::ape::encode::error::encode_error::AtomicValAPEEncodeError;
use crate::constructive::valtype::val::atomic_val::atomic_val::AtomicVal;
use bit_vec::BitVec;

impl AtomicVal {
    /// Airly Payload Encoding (APE) encoding for `AtomicVal`.
    ///
    /// This function encodes an `AtomicVal` into an Airly Payload Encoding (APE) bit vector.
    /// The `AtomicVal` can be a u8, u16, u24, u32, u40, u48, u56, or u64.
    ///
    /// # Arguments
    /// * `&self` - The `AtomicVal` to encode.
    pub fn encode_ape(&self) -> Result<BitVec, AtomicValAPEEncodeError> {
        // 1 Determine the bitsize of the `AtomicVal`.
        let bitsize = AtomicVal::bitsize(self.upper_bound());

        // 2 Convert the value to a n-bit BitVec.
        let bits =
            u8::to_bits(self.value(), bitsize).ok_or(AtomicValAPEEncodeError::U8ExtToBitsError)?;

        // 3 Return the bit vector.
        Ok(bits)
    }
}
